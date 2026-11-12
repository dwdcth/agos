//! 跨主题关联层（图关系存储与查询）。
//!
//! 主路径（`Domain / Topic / Aspect / Kind / Fact` 树）回答"这条记忆挂在哪"，
//! 本模块回答"这条记忆和哪些记忆有关系"——树主路径之外的图关系边：
//!
//! - **存储**：`memory_links` 表，record_id ↔ record_id 的边；`relation` 是
//!   开放字符串（`derived_from` / `relates_to` / `supports` / `contradicts`…），
//!   `weight` 是 [0,1] 的相似/支持强度，`provenance` 记录边的来源
//!   （`provenance_backfill` / `rumination` / `manual`…）。
//! - **边的两个来源**（与差距表对应）：
//!   1. `backfill_from_provenance`：从既有 `provenance.derived_from` 回填——
//!      ingest 写入的派生锚（`uri#anchor`）不是 record id，同锚记录是
//!      "同源共派生"的兄弟块，回填为 `relates_to` 链（按 chunk 顺序相邻连边，
//!      保持稀疏）；若 `derived_from` 直接引用了库内 record id，则回填为
//!      方向保持的 `derived_from` 边。
//!   2. `discover_related`：反刍长周期 pass 的关联发现——BM25 互相命中：
//!      对候选记录两两打 BM25 分（查询方词表 × 文档方词频），互入对方
//!      top-k 且非同源（排除同文档 chunk 的天然相似）→ `relates_to` 边，
//!      weight 取双向归一化分的较小值。
//! - **版本化/衰减**：`valid_from` / `valid_to` 语义照 `memory_records`——
//!   `valid_to IS NULL` 为当前有效；`close_valid_records_by_source` 关闭
//!   记录版本时同步关闭触及该记录的开放边（见 repository 的钩子）。
//! - **查询面**：`neighbors(record_id, depth, relation?)` 供宿主与检索融合
//!   使用（`SearchService` 的邻居扩展默认关，见 `config.graph`）。
//!
//! `contradiction` 检测留给后续（差距表明示）；本层只留关系名与查询面。

use std::collections::{BTreeSet, HashMap, HashSet};

use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::memory::record::{MemoryRecord, ValidityWindow};

/// 对称关系：边无方向语义（存储时按端点字典序规范化，保证 a↔b 只存一行）。
pub const RELATION_RELATES_TO: &str = "relates_to";
/// 方向关系：source 派生自 target。
pub const RELATION_DERIVED_FROM: &str = "derived_from";
/// 方向关系：source 支持 target。
pub const RELATION_SUPPORTS: &str = "supports";
/// 方向关系：source 与 target 冲突（检测留给后续，先占关系名）。
pub const RELATION_CONTRADICTS: &str = "contradicts";

/// 已知关系名（存储层不强制封闭词表，查询侧可传任意非空字符串）。
pub const KNOWN_RELATIONS: &[&str] = &[
    RELATION_RELATES_TO,
    RELATION_DERIVED_FROM,
    RELATION_SUPPORTS,
    RELATION_CONTRADICTS,
];

/// 关联发现 pass 的回填来源标记。
pub const PROVENANCE_PROVENANCE_BACKFILL: &str = "provenance_backfill";
/// 关联发现 pass 的 BM25 互相命中来源标记。
pub const PROVENANCE_RUMINATION: &str = "rumination";

fn is_symmetric_relation(relation: &str) -> bool {
    relation == RELATION_RELATES_TO
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryLink {
    pub link_id: String,
    pub source_record_id: String,
    pub target_record_id: String,
    pub relation: String,
    pub weight: f32,
    pub validity: ValidityWindow,
    pub provenance: String,
    pub created_at: String,
    pub updated_at: String,
}

impl MemoryLink {
    /// 构造一条边；对称关系（`relates_to`）按端点字典序规范化存储方向，
    /// 使 a↔b 与 b↔a 落到同一唯一键。`weight` 收敛到 [0,1]。
    pub fn new(
        source_record_id: impl Into<String>,
        target_record_id: impl Into<String>,
        relation: impl Into<String>,
        weight: f32,
        provenance: impl Into<String>,
        now: impl Into<String>,
    ) -> Self {
        let mut source_record_id = source_record_id.into();
        let mut target_record_id = target_record_id.into();
        let relation = relation.into();
        let provenance = provenance.into();
        if is_symmetric_relation(&relation) && target_record_id < source_record_id {
            std::mem::swap(&mut source_record_id, &mut target_record_id);
        }
        let now = now.into();
        Self {
            link_id: make_link_id(&source_record_id, &target_record_id, &relation, &provenance),
            source_record_id,
            target_record_id,
            relation,
            weight: weight.clamp(0.0, 1.0),
            validity: ValidityWindow {
                valid_from: Some(now.clone()),
                valid_to: None,
            },
            provenance,
            created_at: now.clone(),
            updated_at: now,
        }
    }

    /// 关闭该边（版本下线，不物理删）。
    pub fn closed(mut self, valid_to: impl Into<String>) -> Self {
        self.validity.valid_to = Some(valid_to.into());
        self
    }

    /// 边的另一端。
    pub fn other_end(&self, record_id: &str) -> Option<&str> {
        if self.source_record_id == record_id {
            Some(&self.target_record_id)
        } else if self.target_record_id == record_id {
            Some(&self.source_record_id)
        } else {
            None
        }
    }
}

fn make_link_id(
    source_record_id: &str,
    target_record_id: &str,
    relation: &str,
    provenance: &str,
) -> String {
    let seed = format!("{source_record_id}|{target_record_id}|{relation}|{provenance}");
    let mut hash = 0xcbf29ce484222325u64;
    for byte in seed.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("link-{hash:016x}")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LinkDirection {
    /// 两端都算（默认）。
    #[default]
    Both,
    /// 只算 source_record_id = 查询记录 的出边。
    Outgoing,
    /// 只算 target_record_id = 查询记录 的入边。
    Incoming,
}

/// 边查询条件：关系过滤 + 方向 + 时间窗（valid_to 语义照 record）。
///
/// - `as_of = Some(t)`：取有效期窗口包含 t 的边；
/// - `as_of = None` 且 `include_closed = false`（默认）：只取当前开放边
///   （`valid_to IS NULL`）；
/// - `as_of = None` 且 `include_closed = true`：全部边（含历史）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinkQuery {
    pub relation: Option<String>,
    pub direction: LinkDirection,
    pub as_of: Option<String>,
    pub include_closed: bool,
    pub limit: Option<usize>,
}

impl Default for LinkQuery {
    fn default() -> Self {
        Self {
            relation: None,
            direction: LinkDirection::Both,
            as_of: None,
            include_closed: false,
            limit: None,
        }
    }
}

impl LinkQuery {
    pub fn with_relation(mut self, relation: impl Into<String>) -> Self {
        self.relation = Some(relation.into());
        self
    }

    pub fn with_direction(mut self, direction: LinkDirection) -> Self {
        self.direction = direction;
        self
    }

    pub fn with_as_of(mut self, as_of: impl Into<String>) -> Self {
        self.as_of = Some(as_of.into());
        self
    }

    pub fn with_include_closed(mut self, include_closed: bool) -> Self {
        self.include_closed = include_closed;
        self
    }

    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }
}

/// `neighbors()` 的单跳结果：解析出的邻居记录 + 走过的边。
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryNeighbor {
    pub record: MemoryRecord,
    /// 到达该邻居所经的边（可能有多条路径，只保留首达边）。
    pub link: MemoryLink,
    /// 距起点跳数（1 = 一跳）。
    pub depth: u32,
}

#[derive(Debug, Error)]
pub enum LinkStoreError {
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
    #[error(transparent)]
    Repository(#[from] crate::memory::repository::RepositoryError),
    #[error("link relation must be a non-empty string")]
    InvalidRelation,
}

/// BM25 互相命中关联发现的参数。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LinkDiscoveryOptions {
    /// 参与发现的记录上限（取 `recorded_at` 最新的 N 条，控制 pass 时长）。
    pub max_records: usize,
    /// 每条记录的 BM25 top-k 命中（互相命中的判定窗口）。
    pub top_k: usize,
    /// 归一化相似度下限（低于此值不成边）。
    pub min_score: f32,
}

impl Default for LinkDiscoveryOptions {
    fn default() -> Self {
        Self {
            max_records: 96,
            top_k: 4,
            min_score: 0.35,
        }
    }
}

pub struct MemoryLinkStore<'db> {
    conn: &'db Connection,
}

impl<'db> MemoryLinkStore<'db> {
    pub fn new(conn: &'db Connection) -> Self {
        Self { conn }
    }

    /// 幂等写入：同 (source, target, relation, provenance) 唯一键 upsert。
    pub fn put_link(&self, link: &MemoryLink) -> Result<(), LinkStoreError> {
        if link.relation.trim().is_empty() {
            return Err(LinkStoreError::InvalidRelation);
        }

        self.conn.execute(
            r#"
            INSERT INTO memory_links (
                link_id,
                source_record_id,
                target_record_id,
                relation,
                weight,
                valid_from,
                valid_to,
                provenance,
                created_at,
                updated_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            ON CONFLICT(source_record_id, target_record_id, relation, provenance) DO UPDATE SET
                weight = excluded.weight,
                valid_from = excluded.valid_from,
                valid_to = excluded.valid_to,
                updated_at = excluded.updated_at
            "#,
            params![
                &link.link_id,
                &link.source_record_id,
                &link.target_record_id,
                &link.relation,
                link.weight,
                &link.validity.valid_from,
                &link.validity.valid_to,
                &link.provenance,
                &link.created_at,
                &link.updated_at,
            ],
        )?;

        Ok(())
    }

    /// 只在不存在时插入（回填/发现 pass 用）：已存在则不动，返回是否新插入。
    pub fn insert_link_if_absent(&self, link: &MemoryLink) -> Result<bool, LinkStoreError> {
        if link.relation.trim().is_empty() {
            return Err(LinkStoreError::InvalidRelation);
        }

        let inserted = self.conn.execute(
            r#"
            INSERT OR IGNORE INTO memory_links (
                link_id,
                source_record_id,
                target_record_id,
                relation,
                weight,
                valid_from,
                valid_to,
                provenance,
                created_at,
                updated_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            "#,
            params![
                &link.link_id,
                &link.source_record_id,
                &link.target_record_id,
                &link.relation,
                link.weight,
                &link.validity.valid_from,
                &link.validity.valid_to,
                &link.provenance,
                &link.created_at,
                &link.updated_at,
            ],
        )?;

        Ok(inserted > 0)
    }

    pub fn get_link(
        &self,
        source_record_id: &str,
        target_record_id: &str,
        relation: &str,
        provenance: &str,
    ) -> Result<Option<MemoryLink>, LinkStoreError> {
        self.conn
            .query_row(
                format!(
                    "{LINK_COLUMNS_SELECT} WHERE source_record_id = ?1 AND target_record_id = ?2 \
                     AND relation = ?3 AND provenance = ?4"
                )
                .as_str(),
                params![source_record_id, target_record_id, relation, provenance],
                map_link_row,
            )
            .optional()
            .map_err(Into::into)
    }

    /// 取触及 `record_id` 的边（方向/关系/时间窗由 query 控制）。
    ///
    /// SQL 用统一占位符形态（?2/?3/?5 允许 NULL）避免按选项拼参数个数：
    /// NULL 关系 = 不过滤关系；LIMIT NULL = SQLite 无上限。
    pub fn links_for_record(
        &self,
        record_id: &str,
        query: &LinkQuery,
    ) -> Result<Vec<MemoryLink>, LinkStoreError> {
        let mut sql = format!("{LINK_COLUMNS_SELECT} WHERE ");
        match query.direction {
            LinkDirection::Both => sql.push_str("(source_record_id = ?1 OR target_record_id = ?1)"),
            LinkDirection::Outgoing => sql.push_str("source_record_id = ?1"),
            LinkDirection::Incoming => sql.push_str("target_record_id = ?1"),
        }
        sql.push_str(
            " AND (?2 IS NULL OR relation = ?2) \
              AND ( \
                ?5 = 1 \
                OR (?3 IS NOT NULL AND (valid_from IS NULL OR valid_from <= ?3) \
                    AND (valid_to IS NULL OR valid_to >= ?3)) \
                OR (?3 IS NULL AND valid_to IS NULL) \
              ) \
              ORDER BY weight DESC, link_id ASC \
              LIMIT COALESCE(?4, -1)",
        );

        let mut statement = self.conn.prepare(&sql)?;
        let mut rows = statement.query(params![
            record_id,
            query.relation.as_deref(),
            query.as_of.as_deref(),
            query.limit.map(|limit| limit as i64),
            if query.include_closed { 1i64 } else { 0i64 },
        ])?;

        let mut links = Vec::new();
        while let Some(row) = rows.next()? {
            links.push(map_link_row(row)?);
        }
        Ok(links)
    }

    /// BFS 邻居：按 `query` 允许的边逐跳扩展到 `depth`（1 = 一跳）。
    /// 返回首达路径（每邻居一条），按深度稳定排序。
    pub fn neighbors(
        &self,
        record_id: &str,
        depth: u32,
        query: &LinkQuery,
    ) -> Result<Vec<MemoryNeighbor>, LinkStoreError> {
        let depth = depth.clamp(1, 6);
        let repository = crate::memory::repository::MemoryRepository::new(self.conn);
        let mut neighbors: Vec<MemoryNeighbor> = Vec::new();
        let mut visited: HashSet<String> = HashSet::from([record_id.to_string()]);
        let mut frontier: Vec<String> = vec![record_id.to_string()];

        for hop in 1..=depth {
            if frontier.is_empty() {
                break;
            }
            let mut next_frontier = Vec::new();
            for node in &frontier {
                for link in self.links_for_record(node, query)? {
                    let Some(other) = link.other_end(node) else {
                        continue;
                    };
                    if !visited.insert(other.to_string()) {
                        continue;
                    }
                    // 记录被物理删除（级联删边后理论不可达，防御性跳过）
                    let Some(record) = repository.get_record(other)? else {
                        continue;
                    };
                    next_frontier.push(other.to_string());
                    neighbors.push(MemoryNeighbor {
                        record,
                        link,
                        depth: hop,
                    });
                }
            }
            frontier = next_frontier;
        }

        Ok(neighbors)
    }

    /// 关闭触及 `record_id` 的全部开放边（版本化下线钩子）。返回关闭条数。
    pub fn close_links_touching_record(
        &self,
        record_id: &str,
        valid_to: &str,
    ) -> Result<u64, LinkStoreError> {
        let affected = self.conn.execute(
            r#"
            UPDATE memory_links
            SET valid_to = ?2, updated_at = ?2
            WHERE valid_to IS NULL
              AND (source_record_id = ?1 OR target_record_id = ?1)
            "#,
            params![record_id, valid_to],
        )?;
        Ok(affected as u64)
    }

    /// 从既有 `provenance.derived_from` 数据回填边（幂等，可重复运行）。
    ///
    /// 语义映射（见模块头）：`derived_from` 直接引用库内 record id → 方向
    /// 保持的 `derived_from` 边；指向来源锚（`uri#anchor`）→ 同锚记录是
    /// 同源共派生的兄弟块，按 chunk 顺序相邻连 `relates_to` 链（稀疏，不
    /// 造完全图）。返回新插入的边数。
    pub fn backfill_from_provenance(&self, now: &str) -> Result<usize, LinkStoreError> {
        let repository = crate::memory::repository::MemoryRepository::new(self.conn);
        let records = repository.list_records()?;
        let known_ids: HashSet<&str> = records.iter().map(|record| record.id.as_str()).collect();

        let mut inserted = 0usize;
        let mut anchor_groups: HashMap<&str, Vec<&MemoryRecord>> = HashMap::new();
        for record in &records {
            for derived in &record.provenance.derived_from {
                let derived = derived.trim();
                if derived.is_empty() {
                    continue;
                }
                if known_ids.contains(derived) {
                    if derived != record.id
                        && self.insert_link_if_absent(&MemoryLink::new(
                            &record.id,
                            derived,
                            RELATION_DERIVED_FROM,
                            1.0,
                            PROVENANCE_PROVENANCE_BACKFILL,
                            now,
                        ))?
                    {
                        inserted += 1;
                    }
                } else {
                    anchor_groups.entry(derived).or_default().push(record);
                }
            }
        }

        for (_, group) in anchor_groups {
            if group.len() < 2 {
                continue;
            }
            let mut ordered: Vec<&MemoryRecord> = group;
            ordered.sort_by(|left, right| {
                chunk_sort_key(left)
                    .cmp(&chunk_sort_key(right))
                    .then_with(|| left.id.cmp(&right.id))
            });
            for pair in ordered.windows(2) {
                let (left, right) = (pair[0], pair[1]);
                if self.insert_link_if_absent(&MemoryLink::new(
                    &left.id,
                    &right.id,
                    RELATION_RELATES_TO,
                    1.0,
                    PROVENANCE_PROVENANCE_BACKFILL,
                    now,
                ))? {
                    inserted += 1;
                }
            }
        }

        Ok(inserted)
    }

    /// BM25 互相命中关联发现（反刍长周期 pass 调用）。
    ///
    /// - 候选 = 当前有效（`valid_to IS NULL`）记录中 `recorded_at` 最新
    ///   `max_records` 条；
    /// - 对每个候选以其词表为查询，对其余候选算 BM25 分，取 top-k；
    /// - 互相命中（a∈top(b) 且 b∈top(a)）且非同源（不同 `source_uri`，排除
    ///   同文档 chunk 的天然相似）且归一化分 ≥ `min_score` → `relates_to`；
    /// - 已有任何来源的开放 `relates_to` 边的记录对不重复建边。
    ///
    /// 返回新插入的边数。
    pub fn discover_related(
        &self,
        now: &str,
        options: &LinkDiscoveryOptions,
    ) -> Result<usize, LinkStoreError> {
        let candidates = self.discovery_candidates(options.max_records)?;
        if candidates.len() < 2 {
            return Ok(0);
        }

        let existing_pairs = self.open_relates_to_pairs()?;
        let docs: Vec<DiscoveredDoc> = candidates.iter().map(DiscoveredDoc::tokenize).collect();
        let df = document_frequencies(&docs);
        let avg_len =
            docs.iter().map(|doc| doc.term_counts.len()).sum::<usize>() as f32 / docs.len() as f32;

        let mut top_hits: Vec<BTreeSet<usize>> = Vec::with_capacity(docs.len());
        for (index, doc) in docs.iter().enumerate() {
            let mut scored: Vec<(usize, f32)> = docs
                .iter()
                .enumerate()
                .filter(|(other, _)| {
                    *other != index && candidates[*other].source.uri != candidates[index].source.uri
                })
                .map(|(other, other_doc)| {
                    (other, bm25_score(doc, other_doc, &df, docs.len(), avg_len))
                })
                .filter(|(_, score)| normalized_similarity(*score) >= options.min_score)
                .collect();
            scored.sort_by(|left, right| {
                right
                    .1
                    .total_cmp(&left.1)
                    .then_with(|| left.0.cmp(&right.0))
            });
            scored.truncate(options.top_k.max(1));
            top_hits.push(scored.into_iter().map(|(other, _)| other).collect());
        }

        let mut inserted = 0usize;
        for (a, hits) in top_hits.iter().enumerate() {
            for &b in hits {
                if !top_hits[b].contains(&a) {
                    continue; // 非互相命中
                }
                let (left, right) = (&candidates[a], &candidates[b]);
                let pair = canonical_pair(&left.id, &right.id);
                if existing_pairs.contains(&pair) {
                    continue;
                }
                let weight =
                    normalized_similarity(bm25_score(&docs[a], &docs[b], &df, docs.len(), avg_len))
                        .min(normalized_similarity(bm25_score(
                            &docs[b],
                            &docs[a],
                            &df,
                            docs.len(),
                            avg_len,
                        )))
                        .max(0.05);
                if self.insert_link_if_absent(&MemoryLink::new(
                    &left.id,
                    &right.id,
                    RELATION_RELATES_TO,
                    weight,
                    PROVENANCE_RUMINATION,
                    now,
                ))? {
                    inserted += 1;
                }
            }
        }

        Ok(inserted)
    }

    fn discovery_candidates(
        &self,
        max_records: usize,
    ) -> Result<Vec<MemoryRecord>, LinkStoreError> {
        let repository = crate::memory::repository::MemoryRepository::new(self.conn);
        let mut statement = self.conn.prepare(
            r#"
            SELECT id
            FROM memory_records
            WHERE valid_to IS NULL
            ORDER BY recorded_at DESC, id ASC
            LIMIT ?1
            "#,
        )?;
        let ids = statement
            .query_map([max_records as i64], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;

        ids.iter()
            .map(|id| {
                repository
                    .get_record(id)
                    .map(|record| record.expect("candidate id was just selected"))
            })
            .collect::<Result<Vec<_>, _>>()
            .map_err(LinkStoreError::from)
    }

    fn open_relates_to_pairs(&self) -> Result<HashSet<(String, String)>, LinkStoreError> {
        let mut statement = self.conn.prepare(
            "SELECT source_record_id, target_record_id FROM memory_links \
             WHERE relation = ?1 AND valid_to IS NULL",
        )?;
        let pairs = statement
            .query_map([RELATION_RELATES_TO], |row| {
                Ok(canonical_pair(
                    &row.get::<_, String>(0)?,
                    &row.get::<_, String>(1)?,
                ))
            })?
            .collect::<Result<HashSet<_>, _>>()?;

        Ok(pairs)
    }
}

fn chunk_sort_key(record: &MemoryRecord) -> u32 {
    record
        .chunk
        .as_ref()
        .map(|chunk| chunk.chunk_index)
        .unwrap_or(u32::MAX)
}

fn canonical_pair(a: &str, b: &str) -> (String, String) {
    if a <= b {
        (a.to_string(), b.to_string())
    } else {
        (b.to_string(), a.to_string())
    }
}

const LINK_COLUMNS_SELECT: &str = r#"
    SELECT
        link_id,
        source_record_id,
        target_record_id,
        relation,
        weight,
        valid_from,
        valid_to,
        provenance,
        created_at,
        updated_at
    FROM memory_links
"#;

fn map_link_row(row: &rusqlite::Row<'_>) -> Result<MemoryLink, rusqlite::Error> {
    Ok(MemoryLink {
        link_id: row.get(0)?,
        source_record_id: row.get(1)?,
        target_record_id: row.get(2)?,
        relation: row.get(3)?,
        weight: row.get(4)?,
        validity: ValidityWindow {
            valid_from: row.get(5)?,
            valid_to: row.get(6)?,
        },
        provenance: row.get(7)?,
        created_at: row.get(8)?,
        updated_at: row.get(9)?,
    })
}

// ---------------------------------------------------------------------------
// Rust 侧 BM25（互相命中聚类）
// ---------------------------------------------------------------------------

/// 词频哈希文档：token → 词频。token 化规则与统计分类器一致——ASCII
/// 字母数字串 + CJK 二元组（中文以 bigram 为特征，见 calibrated.rs）。
struct DiscoveredDoc {
    term_counts: HashMap<String, u32>,
}

impl DiscoveredDoc {
    fn tokenize(record: &MemoryRecord) -> Self {
        let text = format!("{} {}", record.source.uri, record.content_text).to_lowercase();
        let mut term_counts: HashMap<String, u32> = HashMap::new();
        for token in crate::memory::calibrated::tokenize_features(&text) {
            *term_counts.entry(token).or_insert(0u32) += 1;
        }
        Self { term_counts }
    }
}

fn document_frequencies(docs: &[DiscoveredDoc]) -> HashMap<String, u32> {
    let mut df: HashMap<String, u32> = HashMap::new();
    for doc in docs {
        for term in doc.term_counts.keys() {
            *df.entry(term.clone()).or_insert(0u32) += 1;
        }
    }
    df
}

/// 标准 BM25（k1=1.2, b=0.75）：以 `query` 的词表查询 `doc` 的词频。
fn bm25_score(
    query: &DiscoveredDoc,
    doc: &DiscoveredDoc,
    df: &HashMap<String, u32>,
    total_docs: usize,
    avg_len: f32,
) -> f32 {
    const K1: f32 = 1.2;
    const B: f32 = 0.75;

    let doc_len = doc.term_counts.values().sum::<u32>() as f32;
    if doc_len == 0.0 || avg_len == 0.0 {
        return 0.0;
    }

    let mut score = 0.0f32;
    for (term, query_tf) in &query.term_counts {
        let Some(doc_tf) = doc.term_counts.get(term) else {
            continue;
        };
        let df_value = df.get(term).copied().unwrap_or(1) as f32;
        let idf = ((total_docs as f32 - df_value + 0.5) / (df_value + 0.5) + 1.0).ln();
        let tf = *doc_tf as f32;
        let denom = tf + K1 * (1.0 - B + B * doc_len / avg_len);
        score += idf * tf * (K1 + 1.0) / denom * (*query_tf as f32).min(3.0).sqrt();
    }
    score
}

/// BM25 原始分（无界，越高越相似）→ [0,1) 归一化。
fn normalized_similarity(raw: f32) -> f32 {
    if raw <= 0.0 { 0.0 } else { raw / (raw + 3.0) }
}
