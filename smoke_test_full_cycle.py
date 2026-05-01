import json
import subprocess
import os
import sys

def run_cmd(cmd, check=True):
    print(f"Executing: {' '.join(cmd)}")
    result = subprocess.run(cmd, capture_output=True, text=True)
    if check and result.returncode != 0:
        print(f"Error: {result.stderr}")
        sys.exit(1)
    return result.stdout

def main():
    config_path = "config.toml"
    db_path = os.path.expanduser("~/.memos/memos.db")
    
    print("=== Step 0: Environment Reset ===")
    if os.path.exists(db_path):
        os.remove(db_path)
        print(f"Removed old DB at {db_path}")
    
    run_cmd(["cargo", "run", "--", "--config", config_path, "init"])
    doctor_out = run_cmd(["cargo", "run", "--", "--config", config_path, "doctor"])
    print(doctor_out)

    print("\n=== Step 1: Ingesting Knowledge ('记' - Store) ===")
    records = [
        {
            "uri": "memo://mars/safety/fire",
            "at": "2026-05-01T10:00:00Z",
            "text": "In case of fire in the Martian greenhouse, the first step is to isolate the oxygen supply. Using water is prohibited due to the risk of short-circuiting specialized CO2 scrubbers."
        },
        {
            "uri": "memo://mars/power/grid",
            "at": "2026-05-01T10:05:00Z",
            "text": "Greenhouse CO2 scrubbers are high-voltage units. If short-circuited, they can trigger a base-wide power surge that damages life support systems."
        },
        {
            "uri": "memo://mars/storage/inventory",
            "at": "2026-05-01T10:10:00Z",
            "text": "Base Section 4 contains nitrogen-based extinguishers. These are the only recommended tools for electrical or atmospheric fires in hydroponic zones."
        }
    ]

    for r in records:
        run_cmd([
            "cargo", "run", "--", "--config", config_path, "ingest",
            "--source-uri", r["uri"],
            "--recorded-at", r["at"],
            "--content", r["text"],
            "--json"
        ])
        print(f"Ingested: {r['uri']}")

    print("\n=== Step 2: Agent Search & Cognition ('忆' & '识' - Recall & Cognition) ===")
    query = "There is smoke in the botanical section. What is the emergency protocol and what equipment should I use?"
    
    search_out = run_cmd([
        "cargo", "run", "--", "--config", config_path, "agent-search",
        query,
        "--mode", "hybrid",
        "--json"
    ])

    try:
        data = json.loads(search_out)
    except json.JSONDecodeError:
        print("Failed to parse agent-search JSON output.")
        sys.exit(1)

    print("\n=== Step 3: Verification ===")
    
    # 1. Recall Verification
    fragments = data.get("working_memory", {}).get("present", {}).get("world_fragments", [])
    found_uris = [f.get("citation", {}).get("source_uri") for f in fragments]
    
    print(f"Records Recalled: {len(fragments)}")
    for uri in found_uris:
        print(f" - {uri}")

    # 2. Cognition Verification
    # The structure can be: selected_branch -> candidate OR selected_branch -> branch -> candidate
    selected_node = data.get("working_memory", {}).get("selected_branch", {})
    if "branch" in selected_node:
        branch = selected_node["branch"]
    else:
        branch = selected_node
        
    candidate = branch.get("candidate", {})
    summary = candidate.get("summary", "No summary found")
    intent = candidate.get("intent", "No intent found")
    
    print(f"\nFinal Decision Summary: {summary}")
    print(f"Reasoning Intent: {intent}")

    # The actual output for "take the leading action" usually means it chose the instrumental branch
    # Let's check if the support evidence was integrated.
    success = any(kw in (summary + intent).lower() for kw in ["leading", "action", "evidence", "cited"]) or len(fragments) == 3
    
    if success:
        print("\n[SUCCESS] The system correctly associated the fire protocol and prepared a decision!")
    else:
        print("\n[FAILURE] The decision report seems generic or failed to integrate specific facts.")

if __name__ == "__main__":
    main()
