//! 可校准的统计分类层（关键词规则之上、LLM 之下的中间层）。
//!
//! 0415 理论的"统计分类器"定位：封闭 taxonomy 中判定
//! `Domain / Topic / Aspect / Kind` 的传统判别器，比 LLM 更适合做主分类器。
//! 当前仓库只有 `KeywordTaxonomyClassifier`（规则，无法从数据校准）；
//! 本模块在其上加一层**可训练的朴素贝叶斯兜底**：
//!
//! - **训练数据** = 既有 DSL sidecar（`fact_dsl_records` 已标
//!   domain/topic/aspect/kind 的记录）——离线、无外部依赖；
//! - **特征** = 词频哈希：ASCII 字母数字串 + CJK 二元组（中文以 bigram
//!   为特征），token 即特征（哈希进计数表，不引外部分词）；
//! - **决策次序**：关键词规则优先（规则命中即用规则输出；命中判定按
//!   内容文本，剥掉 scope/record_type 的恒真泄漏）；规则未命中（无任何
//!   内容关键词命中）→ 统计兜底；统计置信度低于阈值 → 回
//!   `observation/misc` 形状的 fallback（needs_review = true）；
//! - **默认关**：`[memory] calibrated_classifier = false`，ingest 不受影响；
//!   CLI `agent-memos calibrate` 离线训练 + 留出评估（准确率/覆盖度报告）。
//!
//! 训练样本不足（< `MIN_TRAINING_RECORDS`）时统计层不可用，行为与纯关键词
//! 分类器一致——宿主接线零风险。

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::memory::classifier::{
    ClassificationError, ClassificationInput, ClassificationOutput, KeywordClassificationReport,
    KeywordTaxonomyClassifier, TaxonomyClassifier,
};
use crate::memory::record::MemoryRecord;
use crate::memory::repository::{LayeredMemoryRecord, MemoryRepository};
use crate::memory::taxonomy::{AspectV1, DomainV1, KindV1, TaxonomyPathV1, TopicV1};

/// 训练样本少于该数时统计层不启用（避免 1-2 条记录上的过拟合兜底）。
pub const MIN_TRAINING_RECORDS: usize = 4;

/// 规则探针的中性 source_ref：分词后不含任何关键词表的词（`memo://` 会
/// 恒命中 memory 主题、`roadmap://` 会恒命中 project 域，均不可用）。
const CONTENT_RULE_PROBE_SOURCE_REF: &str = "probe://zzz";

/// 特征化：ASCII 字母数字串原样（转小写），连续 CJK 字符切二元组
/// （单字保留为一元组）。中文无空部分词，bigram 是无依赖的最小可用特征。
pub fn tokenize_features(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut ascii_run = String::new();
    let mut cjk_run: Vec<char> = Vec::new();

    let flush_ascii = |run: &mut String, tokens: &mut Vec<String>| {
        if !run.is_empty() {
            tokens.push(std::mem::take(run));
        }
    };
    let flush_cjk = |run: &mut Vec<char>, tokens: &mut Vec<String>| match run.len() {
        0 => {}
        1 => {
            tokens.push(run[0].to_string());
            run.clear();
        }
        _ => {
            for pair in run.windows(2) {
                tokens.push(pair.iter().collect());
            }
            run.clear();
        }
    };

    for ch in text.chars() {
        if ch.is_ascii_alphanumeric() {
            flush_cjk(&mut cjk_run, &mut tokens);
            ascii_run.push(ch.to_ascii_lowercase());
        } else if is_cjk(ch) {
            flush_ascii(&mut ascii_run, &mut tokens);
            cjk_run.push(ch);
        } else {
            flush_ascii(&mut ascii_run, &mut tokens);
            flush_cjk(&mut cjk_run, &mut tokens);
        }
    }
    flush_ascii(&mut ascii_run, &mut tokens);
    flush_cjk(&mut cjk_run, &mut tokens);

    tokens
}

fn is_cjk(ch: char) -> bool {
    matches!(ch as u32,
        0x3400..=0x4DBF   // CJK 扩展 A
        | 0x4E00..=0x9FFF // CJK 基本
        | 0xF900..=0xFAFF // CJK 兼容
    )
}

/// 一条带标注的训练样本（来自 DSL sidecar）。`source_ref` 与 `text`
/// 分开存：统计层特征用两者拼接（URI 方案含语义信号），规则探针只用
/// `text`（见 [`CONTENT_RULE_PROBE_SOURCE_REF`] 的说明）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CalibratedTrainingSample {
    pub source_ref: String,
    pub text: String,
    pub domain: DomainV1,
    pub topic: TopicV1,
    pub aspect: AspectV1,
    pub kind: KindV1,
}

impl CalibratedTrainingSample {
    pub fn from_layered_record(record: &LayeredMemoryRecord) -> Option<Self> {
        let dsl = record.dsl.as_ref()?;
        let payload = &dsl.payload;
        Some(Self {
            source_ref: record.record.source.uri.clone(),
            text: record.record.content_text.clone(),
            domain: DomainV1::parse(&payload.domain)?,
            topic: TopicV1::parse(&payload.topic)?,
            aspect: AspectV1::parse(&payload.aspect)?,
            kind: KindV1::parse(&payload.kind)?,
        })
    }

    fn feature_text(&self) -> String {
        format!("{} {}", self.source_ref, self.text).to_lowercase()
    }
}

/// 单字段朴素贝叶斯模型：类先验 + 词频多项式似然（拉普拉斯平滑）。
#[derive(Debug, Clone, PartialEq)]
struct NaiveBayesField {
    class_doc_counts: HashMap<String, u32>,
    class_token_totals: HashMap<String, u32>,
    class_token_counts: HashMap<String, HashMap<String, u32>>,
    vocabulary: std::collections::HashSet<String>,
}

impl NaiveBayesField {
    fn new() -> Self {
        Self {
            class_doc_counts: HashMap::new(),
            class_token_totals: HashMap::new(),
            class_token_counts: HashMap::new(),
            vocabulary: std::collections::HashSet::new(),
        }
    }

    fn observe(&mut self, class: &str, tokens: &[String]) {
        *self.class_doc_counts.entry(class.to_string()).or_insert(0) += 1;
        let counts = self
            .class_token_counts
            .entry(class.to_string())
            .or_default();
        for token in tokens {
            *counts.entry(token.clone()).or_insert(0) += 1;
            self.vocabulary.insert(token.clone());
        }
        *self
            .class_token_totals
            .entry(class.to_string())
            .or_insert(0) += tokens.len() as u32;
    }

    /// 返回 (最优类, 后验置信度 in (0,1])；无训练数据返回 None。
    fn predict(&self, tokens: &[String]) -> Option<(String, f32)> {
        if self.class_doc_counts.is_empty() {
            return None;
        }
        let total_docs: u32 = self.class_doc_counts.values().sum();
        let vocab_size = self.vocabulary.len() as f32;

        let mut best: Option<(String, f32)> = None;
        let mut total_weight = 0.0f32;
        let mut weights = Vec::with_capacity(self.class_doc_counts.len());
        for (class, doc_count) in &self.class_doc_counts {
            let prior = (*doc_count as f32 + 1.0)
                / (total_docs as f32 + self.class_doc_counts.len() as f32);
            let mut log_likelihood = prior.ln();
            let token_total = self.class_token_totals.get(class).copied().unwrap_or(0) as f32;
            let counts = self.class_token_counts.get(class);
            for token in tokens {
                let count = counts
                    .and_then(|counts| counts.get(token))
                    .copied()
                    .unwrap_or(0) as f32;
                log_likelihood += ((count + 1.0) / (token_total + vocab_size)).ln();
            }
            let weight = log_likelihood.exp();
            weights.push(weight);
            total_weight += weight;
            if best
                .as_ref()
                .is_none_or(|(_, best_weight)| weight > *best_weight)
            {
                best = Some((class.clone(), weight));
            }
        }

        best.map(|(class, weight)| {
            let confidence = if total_weight > 0.0 {
                weight / total_weight
            } else {
                0.0
            };
            (class, confidence.clamp(0.0, 1.0))
        })
    }
}

/// 关键词规则 + 统计兜底的复合分类器。
#[derive(Debug, Clone)]
pub struct CalibratedTaxonomyClassifier {
    keyword: KeywordTaxonomyClassifier,
    domain_model: NaiveBayesField,
    topic_model: NaiveBayesField,
    aspect_model: NaiveBayesField,
    kind_model: NaiveBayesField,
    /// 统计层可用的最低训练样本数门槛（构造时判定，存结果）。
    statistical_available: bool,
    /// 统计输出必须超过该后验置信度才被采用，否则回 fallback。
    statistical_confidence_threshold: f32,
}

/// 统计层兜底的缺省置信度门槛：朴素贝叶斯后验在类多/样本少时偏自信，
/// 0.55 只挡掉几乎无信号的预测。
pub const DEFAULT_STATISTICAL_CONFIDENCE_THRESHOLD: f32 = 0.55;

impl CalibratedTaxonomyClassifier {
    /// 从标注样本训练；样本不足时统计层不可用（等价纯关键词分类器）。
    pub fn fit(samples: &[CalibratedTrainingSample]) -> Self {
        Self::fit_with_options(samples, DEFAULT_STATISTICAL_CONFIDENCE_THRESHOLD)
    }

    pub fn fit_with_options(
        samples: &[CalibratedTrainingSample],
        statistical_confidence_threshold: f32,
    ) -> Self {
        let mut classifier = Self {
            keyword: KeywordTaxonomyClassifier::default(),
            domain_model: NaiveBayesField::new(),
            topic_model: NaiveBayesField::new(),
            aspect_model: NaiveBayesField::new(),
            kind_model: NaiveBayesField::new(),
            statistical_available: samples.len() >= MIN_TRAINING_RECORDS,
            statistical_confidence_threshold,
        };
        for sample in samples {
            let tokens = tokenize_features(&sample.feature_text());
            classifier
                .domain_model
                .observe(sample.domain.as_str(), &tokens);
            classifier
                .topic_model
                .observe(sample.topic.as_str(), &tokens);
            classifier
                .aspect_model
                .observe(sample.aspect.as_str(), &tokens);
            classifier.kind_model.observe(sample.kind.as_str(), &tokens);
        }
        classifier
    }

    /// 从库内 DSL sidecar（layered records）收集样本并训练。
    pub fn fit_from_repository(
        repository: &MemoryRepository<'_>,
    ) -> Result<Self, crate::memory::repository::RepositoryError> {
        let records = repository.list_layered_records()?;
        let samples = records
            .iter()
            .filter_map(CalibratedTrainingSample::from_layered_record)
            .collect::<Vec<_>>();
        Ok(Self::fit(&samples))
    }

    pub fn statistical_available(&self) -> bool {
        self.statistical_available
    }

    /// 分类 + 分层报告（哪一层做的决定）。
    ///
    /// 规则层的命中判定是**内容导向**的：`ClassificationInput::from_record`
    /// 拼进关键词文本的 `scope`/`record_type`（scope=project 恒命中 project
    /// 域关键词）与来源 URI scheme（`memo://` 恒命中 memory 主题关键词）
    /// 都不是内容信号——不剥掉它们统计层将永远不被咨询。因此规则探针用
    /// 中性占位 source_ref + 纯正文跑关键词规则；关键词命中时输出也以
    /// 内容规则为准（默认纯关键词路径不受影响，仅本层语义如此）。
    pub fn classify_with_report_sync(
        &self,
        input: &ClassificationInput,
    ) -> Result<(ClassificationOutput, CalibratedClassificationReport), ClassificationError> {
        input.validate()?;

        let content_input = ClassificationInput {
            source_ref: CONTENT_RULE_PROBE_SOURCE_REF.to_string(),
            raw_text: input.raw_text.clone(),
            source_kind: input.source_kind.clone(),
            scope: None,
            record_type: None,
        };
        let (keyword_output, keyword_report) =
            self.keyword.classify_with_report_sync(&content_input)?;
        let rule_hit = keyword_report.has_any_hit();

        if rule_hit {
            return Ok((
                keyword_output,
                CalibratedClassificationReport {
                    layer: CalibratedLayer::Keyword,
                    keyword_report,
                    statistical_confidence: None,
                },
            ));
        }

        if !self.statistical_available {
            return Ok((
                keyword_output,
                CalibratedClassificationReport {
                    layer: CalibratedLayer::KeywordFallback,
                    keyword_report,
                    statistical_confidence: None,
                },
            ));
        }

        let text = format!("{} {}", input.source_ref, input.raw_text).to_lowercase();
        let tokens = tokenize_features(&text);
        let domain = self.domain_model.predict(&tokens);
        let topic = self.topic_model.predict(&tokens);
        let aspect = self.aspect_model.predict(&tokens);
        let kind = self.kind_model.predict(&tokens);

        let confidences = [&domain, &topic, &aspect, &kind]
            .into_iter()
            .filter_map(|prediction| prediction.as_ref().map(|(_, confidence)| *confidence))
            .collect::<Vec<_>>();
        let combined = if confidences.is_empty() {
            0.0
        } else {
            confidences.iter().sum::<f32>() / confidences.len() as f32
        };

        if combined < self.statistical_confidence_threshold || domain.is_none() || topic.is_none() {
            // 低置信：回 observation/misc 形状（保守入库，needs_review=true）
            let fallback = ClassificationOutput::fallback_observation(self.keyword.default_domain);
            return Ok((
                fallback,
                CalibratedClassificationReport {
                    layer: CalibratedLayer::StatisticalFallback,
                    keyword_report,
                    statistical_confidence: Some(combined),
                },
            ));
        }

        let domain = DomainV1::parse(&domain.expect("checked above").0)
            .unwrap_or(self.keyword.default_domain);
        let topic = TopicV1::parse(&topic.expect("checked above").0).unwrap_or_else(|| {
            TopicV1::allowed_for(domain)
                .first()
                .copied()
                .unwrap_or(TopicV1::General)
        });
        let aspect = aspect
            .as_ref()
            .and_then(|(value, _)| AspectV1::parse(value))
            .unwrap_or(AspectV1::General);
        let kind = kind
            .as_ref()
            .and_then(|(value, _)| KindV1::parse(value))
            .unwrap_or(KindV1::Observation);

        let taxonomy = TaxonomyPathV1::new(domain, topic, aspect, kind)?;
        // topic 落到 allowed 之外的兜底（parse 成功但 domain 变了）
        let taxonomy = if taxonomy.validate().is_ok() {
            taxonomy
        } else {
            TaxonomyPathV1::new(domain, TopicV1::General, aspect, kind)?
        };

        let output = ClassificationOutput::new(
            taxonomy,
            combined.clamp(0.0, 1.0),
            combined < self.keyword.review_threshold,
        )?;

        Ok((
            output,
            CalibratedClassificationReport {
                layer: CalibratedLayer::Statistical,
                keyword_report,
                statistical_confidence: Some(combined),
            },
        ))
    }

    pub fn classify_sync(
        &self,
        input: &ClassificationInput,
    ) -> Result<ClassificationOutput, ClassificationError> {
        Ok(self.classify_with_report_sync(input)?.0)
    }

    pub fn classify_record_sync(
        &self,
        record: &MemoryRecord,
    ) -> Result<ClassificationOutput, ClassificationError> {
        let input = ClassificationInput::from_record(record);
        self.classify_sync(&input)
    }
}

impl TaxonomyClassifier for CalibratedTaxonomyClassifier {
    fn classify<'a>(
        &'a self,
        input: &'a ClassificationInput,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<ClassificationOutput, ClassificationError>>
                + Send
                + 'a,
        >,
    > {
        Box::pin(async move { self.classify_sync(input) })
    }
}

/// 分类决定来自哪一层。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CalibratedLayer {
    /// 关键词规则命中。
    Keyword,
    /// 规则未命中且统计层不可用（样本不足）——保持原关键词输出。
    KeywordFallback,
    /// 统计层兜底成功。
    Statistical,
    /// 统计层置信度不足，回 observation/misc fallback。
    StatisticalFallback,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalibratedClassificationReport {
    pub layer: CalibratedLayer,
    pub keyword_report: KeywordClassificationReport,
    /// 统计层触发时的平均后验置信度。
    pub statistical_confidence: Option<f32>,
}

impl KeywordClassificationReport {
    fn has_any_hit(&self) -> bool {
        !self.domain_hits.is_empty()
            || !self.topic_hits.is_empty()
            || !self.aspect_hits.is_empty()
            || !self.kind_hits.is_empty()
    }
}

// ---------------------------------------------------------------------------
// 离线校准评估（CLI `calibrate` 与测试共用）
// ---------------------------------------------------------------------------

/// 一条评估样本：带期望标注的输入。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CalibratedEvaluationSample {
    pub input: ClassificationInput,
    pub expected: TaxonomyPathV1,
}

/// 留出评估报告：准确率（分层）+ 覆盖度。
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CalibratieReport {
    pub total_records: usize,
    pub train_records: usize,
    pub test_records: usize,
    /// 关键词层命中的测试样本占比（规则覆盖度）。
    pub keyword_coverage: f32,
    /// 统计层实际兜底（含低置信 fallback）的占比。
    pub statistical_coverage: f32,
    /// 整管线 domain 准确率。
    pub domain_accuracy: f32,
    pub topic_accuracy: f32,
    pub aspect_accuracy: f32,
    pub kind_accuracy: f32,
    /// 只统计统计层被咨询（规则未命中）样本上的 domain 准确率。
    pub statistical_domain_accuracy: f32,
}

impl CalibratieReport {
    pub fn render_text(&self) -> String {
        format!(
            concat!(
                "records: total={} train={} test={}\n",
                "coverage: keyword={:.1}% statistical={:.1}%\n",
                "accuracy: domain={:.1}% topic={:.1}% aspect={:.1}% kind={:.1}%\n",
                "statistical-layer domain accuracy: {:.1}% (rule-miss subset)"
            ),
            self.total_records,
            self.train_records,
            self.test_records,
            self.keyword_coverage * 100.0,
            self.statistical_coverage * 100.0,
            self.domain_accuracy * 100.0,
            self.topic_accuracy * 100.0,
            self.aspect_accuracy * 100.0,
            self.kind_accuracy * 100.0,
            self.statistical_domain_accuracy * 100.0,
        )
    }
}

/// 留出评估：按 record id 哈希确定性切分 train/test（同数据同切分），
/// 训练 → 在 test 上跑整管线（关键词优先 → 统计兜底 → fallback）。
/// test 样本数 < 1 时各指标为 0。
pub fn evaluate_calibrated_classifier(
    samples: &[CalibratedTrainingSample],
    test_ratio: f32,
) -> CalibratieReport {
    let test_ratio = test_ratio.clamp(0.1, 0.5);
    let mut train = Vec::new();
    let mut test = Vec::new();
    for sample in samples {
        if hash_bucket(&sample.feature_text()) < test_ratio {
            test.push(sample.clone());
        } else {
            train.push(sample.clone());
        }
    }

    let mut report = CalibratieReport {
        total_records: samples.len(),
        train_records: train.len(),
        test_records: test.len(),
        ..CalibratieReport::default()
    };
    if test.is_empty() {
        return report;
    }

    let classifier = CalibratedTaxonomyClassifier::fit(&train);
    let mut keyword_hits = 0usize;
    let mut statistical_consulted = 0usize;
    let mut domain_correct = 0usize;
    let mut topic_correct = 0usize;
    let mut aspect_correct = 0usize;
    let mut kind_correct = 0usize;
    let mut statistical_domain_correct = 0usize;

    for sample in &test {
        let input = ClassificationInput {
            source_ref: sample.source_ref.clone(),
            raw_text: sample.text.clone(),
            source_kind: None,
            scope: None,
            record_type: None,
        };
        let Ok((output, layer_report)) = classifier.classify_with_report_sync(&input) else {
            continue;
        };
        if matches!(layer_report.layer, CalibratedLayer::Keyword) {
            keyword_hits += 1;
        } else {
            statistical_consulted += 1;
            if output.taxonomy.domain == sample.domain {
                statistical_domain_correct += 1;
            }
        }
        if output.taxonomy.domain == sample.domain {
            domain_correct += 1;
        }
        if output.taxonomy.topic == sample.topic {
            topic_correct += 1;
        }
        if output.taxonomy.aspect == sample.aspect {
            aspect_correct += 1;
        }
        if output.taxonomy.kind == sample.kind {
            kind_correct += 1;
        }
    }

    let total = test.len() as f32;
    report.keyword_coverage = keyword_hits as f32 / total;
    report.statistical_coverage = statistical_consulted as f32 / total;
    report.domain_accuracy = domain_correct as f32 / total;
    report.topic_accuracy = topic_correct as f32 / total;
    report.aspect_accuracy = aspect_correct as f32 / total;
    report.kind_accuracy = kind_correct as f32 / total;
    report.statistical_domain_accuracy = if statistical_consulted > 0 {
        statistical_domain_correct as f32 / statistical_consulted as f32
    } else {
        0.0
    };
    report
}

/// 稳定哈希切分桶：[0,1)。
fn hash_bucket(text: &str) -> f32 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in text.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    ((hash % 10_000) as f32) / 10_000.0
}

#[derive(Debug, Error)]
pub enum CalibratedError {
    #[error(transparent)]
    Repository(#[from] crate::memory::repository::RepositoryError),
    #[error(transparent)]
    Classification(#[from] ClassificationError),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(text: &str, domain: DomainV1, topic: TopicV1) -> CalibratedTrainingSample {
        CalibratedTrainingSample {
            source_ref: format!("memo://calibrate/{domain}"),
            text: text.to_string(),
            domain,
            topic,
            aspect: AspectV1::Behavior,
            kind: KindV1::Fact,
        }
    }

    #[test]
    fn tokenize_features_splits_ascii_and_cjk_bigrams() {
        let tokens = tokenize_features("SQLite 数据库 retrieval 基线");
        assert!(tokens.contains(&"sqlite".to_string()));
        assert!(tokens.contains(&"retrieval".to_string()));
        assert!(tokens.contains(&"数据".to_string()));
        assert!(tokens.contains(&"据库".to_string()));
        assert!(tokens.contains(&"基线".to_string()));
    }

    #[test]
    fn statistical_layer_recovers_unseen_domain_from_vocabulary() {
        let samples = vec![
            sample(
                "kubernetes cluster rollout pager duty incident",
                DomainV1::External,
                TopicV1::Provider,
            ),
            sample(
                "kubernetes cluster upgrade vendor api quota",
                DomainV1::External,
                TopicV1::Api,
            ),
            sample(
                "vendor pricing kubernetes cluster cost budget",
                DomainV1::External,
                TopicV1::Cost,
            ),
            sample(
                "lexical retrieval baseline fts5 sqlite search",
                DomainV1::Project,
                TopicV1::Retrieval,
            ),
            sample(
                "lexical retrieval hybrid embedding sqlite",
                DomainV1::Project,
                TopicV1::Retrieval,
            ),
            sample(
                "working memory assembly agent follow-up",
                DomainV1::Project,
                TopicV1::Agent,
            ),
        ];
        let classifier = CalibratedTaxonomyClassifier::fit(&samples);
        assert!(classifier.statistical_available());

        // 无关键词命中（规则 miss，正文不含任何关键词表词）→ 统计层兜底识别 domain
        let (output, report) = classifier
            .classify_with_report_sync(&ClassificationInput::new(
                "memo://calibrate/1",
                "kubernetes cluster fleet quota incident response window",
            ))
            .expect("classify");
        assert_eq!(report.layer, CalibratedLayer::Statistical);
        assert_eq!(output.taxonomy.domain, DomainV1::External);
    }

    #[test]
    fn keyword_layer_takes_priority_when_rules_hit() {
        let samples = vec![
            sample(
                "kubernetes vendor quota",
                DomainV1::External,
                TopicV1::Provider,
            ),
            sample(
                "lexical retrieval baseline sqlite",
                DomainV1::Project,
                TopicV1::Retrieval,
            ),
            sample(
                "lexical retrieval hybrid embedding",
                DomainV1::Project,
                TopicV1::Retrieval,
            ),
            sample(
                "kubernetes cluster vendor api cost",
                DomainV1::External,
                TopicV1::Api,
            ),
        ];
        let classifier = CalibratedTaxonomyClassifier::fit(&samples);

        let (output, report) = classifier
            .classify_with_report_sync(&ClassificationInput::new(
                "roadmap#phase9",
                "Use lexical-first retrieval as the baseline decision because explainability matters.",
            ))
            .expect("classify");
        assert_eq!(report.layer, CalibratedLayer::Keyword);
        assert_eq!(output.taxonomy.topic, TopicV1::Retrieval);
    }

    #[test]
    fn insufficient_samples_disable_statistical_layer() {
        let classifier = CalibratedTaxonomyClassifier::fit(&[sample(
            "only one sample",
            DomainV1::Project,
            TopicV1::Memory,
        )]);
        assert!(!classifier.statistical_available());

        let (_, report) = classifier
            .classify_with_report_sync(&ClassificationInput::new(
                "memo://x",
                "zzz qqq completely unseen vocabulary",
            ))
            .expect("classify");
        assert_eq!(report.layer, CalibratedLayer::KeywordFallback);
    }

    #[test]
    fn evaluation_report_reports_accuracy_and_coverage() {
        let mut samples = Vec::new();
        for i in 0..6 {
            samples.push(sample(
                &format!("kubernetes cluster vendor quota incident {i}"),
                DomainV1::External,
                TopicV1::Provider,
            ));
            samples.push(sample(
                &format!("lexical retrieval baseline sqlite fts {i}"),
                DomainV1::Project,
                TopicV1::Retrieval,
            ));
        }
        let report = evaluate_calibrated_classifier(&samples, 0.3);
        assert_eq!(report.total_records, 12);
        assert!(report.train_records + report.test_records == 12);
        assert!(report.test_records > 0);
        assert!(report.domain_accuracy > 0.0, "合成可分数据应有一定准确率");
    }
}
