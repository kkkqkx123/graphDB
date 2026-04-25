use colored::Colorize;

pub fn show_help(topic: Option<&str>) -> String {
    match topic {
        None => show_general_help(),
        Some(t) => show_topic_help(t),
    }
}

fn show_general_help() -> String {
    let mut output = String::new();

    output.push_str(&format!(
        "\n{}\n",
        "GraphDB CLI - Meta Commands".cyan().bold()
    ));
    output.push_str(&"─".repeat(50).dimmed());
    output.push('\n');

    output.push_str(&format!("\n{}\n", "Connection".yellow().bold()));
    output.push_str(&format!(
        "  {:25} {}\n",
        "\\connect <space>", "Connect to a graph space"
    ));
    output.push_str(&format!(
        "  {:25} {}\n",
        "\\disconnect", "Disconnect from current session"
    ));
    output.push_str(&format!(
        "  {:25} {}\n",
        "\\conninfo", "Display connection information"
    ));

    output.push_str(&format!("\n{}\n", "Object Inspection".yellow().bold()));
    output.push_str(&format!(
        "  {:25} {}\n",
        "\\show_spaces  or \\l", "List all graph spaces"
    ));
    output.push_str(&format!(
        "  {:25} {}\n",
        "\\show_tags   or \\dt", "List all tags (vertex types)"
    ));
    output.push_str(&format!(
        "  {:25} {}\n",
        "\\show_edges  or \\de", "List all edge types"
    ));
    output.push_str(&format!(
        "  {:25} {}\n",
        "\\show_indexes or \\di", "List all indexes"
    ));
    output.push_str(&format!(
        "  {:25} {}\n",
        "\\show_users  or \\du", "List all users"
    ));
    output.push_str(&format!(
        "  {:25} {}\n",
        "\\show_functions or \\df", "List all functions"
    ));
    output.push_str(&format!(
        "  {:25} {}\n",
        "\\describe <tag>", "Describe tag structure"
    ));
    output.push_str(&format!(
        "  {:25} {}\n",
        "\\describe_edge <edge>", "Describe edge type structure"
    ));

    output.push_str(&format!("\n{}\n", "Output Format".yellow().bold()));
    output.push_str(&format!(
        "  {:25} {}\n",
        "\\format <fmt>", "Set output format (table, csv, json, vertical, html)"
    ));
    output.push_str(&format!(
        "  {:25} {}\n",
        "\\pager [cmd]", "Set or disable pager"
    ));
    output.push_str(&format!(
        "  {:25} {}\n",
        "\\timing", "Toggle query execution time display"
    ));
    output.push_str(&format!(
        "  {:25} {}\n",
        "\\x", "Toggle expanded/vertical display"
    ));

    output.push_str(&format!("\n{}\n", "Variables".yellow().bold()));
    output.push_str(&format!(
        "  {:25} {}\n",
        "\\set [name [value]]", "Set or show variables"
    ));
    output.push_str(&format!(
        "  {:25} {}\n",
        "\\unset <name>", "Delete a variable"
    ));

    output.push_str(&format!("\n{}\n", "Script and I/O".yellow().bold()));
    output.push_str(&format!(
        "  {:25} {}\n",
        "\\i <file>", "Execute commands from file"
    ));
    output.push_str(&format!(
        "  {:25} {}\n",
        "\\ir <file>", "Execute commands from file (raw, no substitution)"
    ));
    output.push_str(&format!(
        "  {:25} {}\n",
        "\\o [file]", "Redirect output to file (or close)"
    ));
    output.push_str(&format!(
        "  {:25} {}\n",
        "\\! <command>", "Execute a shell command"
    ));

    output.push_str(&format!("\n{}\n", "Query Buffer".yellow().bold()));
    output.push_str(&format!(
        "  {:25} {}\n",
        "\\e [file] [+line]", "Edit query buffer in external editor"
    ));
    output.push_str(&format!(
        "  {:25} {}\n",
        "\\p", "Print the current query buffer"
    ));
    output.push_str(&format!(
        "  {:25} {}\n",
        "\\r", "Reset (clear) the query buffer"
    ));
    output.push_str(&format!(
        "  {:25} {}\n",
        "\\w <file>", "Write query buffer to file"
    ));

    output.push_str(&format!("\n{}\n", "History".yellow().bold()));
    output.push_str(&format!(
        "  {:25} {}\n",
        "\\history [N]", "Show last N history entries (default 20)"
    ));
    output.push_str(&format!(
        "  {:25} {}\n",
        "\\history search <pat>", "Search history for pattern"
    ));
    output.push_str(&format!(
        "  {:25} {}\n",
        "\\history exec <id>", "Re-execute history entry by ID"
    ));
    output.push_str(&format!(
        "  {:25} {}\n",
        "\\history clear", "Clear command history"
    ));

    output.push_str(&format!("\n{}\n", "Transaction".yellow().bold()));
    output.push_str(&format!("  {:25} {}\n", "\\begin", "Begin a transaction"));
    output.push_str(&format!(
        "  {:25} {}\n",
        "\\commit", "Commit current transaction"
    ));
    output.push_str(&format!(
        "  {:25} {}\n",
        "\\rollback", "Rollback current transaction"
    ));

    output.push_str(&format!("\n{}\n", "Conditional Execution".yellow().bold()));
    output.push_str(&format!(
        "  {:25} {}\n",
        "\\if <condition>", "Begin conditional block"
    ));
    output.push_str(&format!(
        "  {:25} {}\n",
        "\\elif <condition>", "Else-if branch"
    ));
    output.push_str(&format!("  {:25} {}\n", "\\else", "Else branch"));
    output.push_str(&format!("  {:25} {}\n", "\\endif", "End conditional block"));

    output.push_str(&format!("\n{}\n", "General".yellow().bold()));
    output.push_str(&format!(
        "  {:25} {}\n",
        "\\help [command]", "Show help on GQL command"
    ));
    output.push_str(&format!(
        "  {:25} {}\n",
        "\\version", "Show version information"
    ));
    output.push_str(&format!(
        "  {:25} {}\n",
        "\\copyright", "Show copyright information"
    ));
    output.push_str(&format!("  {:25} {}\n", "\\q", "Quit GraphDB CLI"));

    output.push('\n');
    output
}

fn show_topic_help(topic: &str) -> String {
    match topic.to_lowercase().as_str() {
        "match" => {
            let mut s = String::new();
            s.push_str("MATCH statement - Pattern matching query\n\n");
            s.push_str("Syntax:\n");
            s.push_str("  MATCH (v:Tag)\n");
            s.push_str("  WHERE v.property > value\n");
            s.push_str("  RETURN v.property\n\n");
            s.push_str("Examples:\n");
            s.push_str("  MATCH (p:person) RETURN p.name, p.age\n");
            s.push_str("  MATCH (p:person)-[:friend]->(f:person) RETURN p, f\n");
            s
        }
        "go" => {
            let mut s = String::new();
            s.push_str("GO statement - Graph traversal query\n\n");
            s.push_str("Syntax:\n");
            s.push_str("  GO <steps> STEPS FROM <vid> OVER <edge_type>\n");
            s.push_str("  YIELD properties\n\n");
            s.push_str("Examples:\n");
            s.push_str("  GO 1 STEPS FROM \"person1\" OVER friend YIELD friend.name\n");
            s
        }
        "insert" => {
            let mut s = String::new();
            s.push_str("INSERT statement - Insert data\n\n");
            s.push_str("Insert vertex:\n");
            s.push_str("  INSERT VERTEX tag(prop1, prop2) VALUES \"vid\":(val1, val2)\n\n");
            s.push_str("Insert edge:\n");
            s.push_str("  INSERT EDGE edge_type(prop1) VALUES \"src\"->\"dst\":(val1)\n");
            s
        }
        "create" => {
            let mut s = String::new();
            s.push_str("CREATE statement - Schema definition\n\n");
            s.push_str("Create space:\n");
            s.push_str("  CREATE SPACE space_name (vid_type=STRING)\n\n");
            s.push_str("Create tag:\n");
            s.push_str("  CREATE TAG tag_name (prop1 type, prop2 type)\n\n");
            s.push_str("Create edge:\n");
            s.push_str("  CREATE EDGE edge_name (prop1 type)\n");
            s
        }
        "show" => {
            let mut s = String::new();
            s.push_str("SHOW statement - Display metadata\n\n");
            s.push_str("Commands:\n");
            s.push_str("  SHOW SPACES          - List all graph spaces\n");
            s.push_str("  SHOW TAGS            - List all tags in current space\n");
            s.push_str("  SHOW EDGES           - List all edge types in current space\n");
            s.push_str("  SHOW INDEXES         - List all indexes\n");
            s.push_str("  SHOW CREATE TAG <n>  - Show tag creation statement\n");
            s
        }
        "use" => {
            let mut s = String::new();
            s.push_str("USE statement - Switch to a graph space\n\n");
            s.push_str("Syntax:\n");
            s.push_str("  USE space_name\n\n");
            s.push_str("Example:\n");
            s.push_str("  USE my_graph\n");
            s
        }
        "variables" | "set" => {
            let mut s = String::new();
            s.push_str("Variable Management\n\n");
            s.push_str("Set a variable:\n");
            s.push_str("  \\set NAME VALUE\n\n");
            s.push_str("Show a variable:\n");
            s.push_str("  \\set NAME\n\n");
            s.push_str("Show all variables:\n");
            s.push_str("  \\set\n\n");
            s.push_str("Delete a variable:\n");
            s.push_str("  \\unset NAME\n\n");
            s.push_str("Use variables in queries:\n");
            s.push_str("  MATCH (p:person) WHERE p.age > :min_age RETURN p\n");
            s.push_str("  MATCH (p:person) WHERE p.name = :'name' RETURN p\n\n");
            s.push_str("Special variables (marked with *):\n");
            s.push_str("  ON_ERROR_STOP  - Stop on error (on/off)\n");
            s.push_str("  ECHO           - Echo mode (none/queries/all)\n");
            s.push_str("  TIMING         - Show execution time (on/off)\n");
            s.push_str("  EDITOR         - External editor command\n");
            s.push_str("  FORMAT         - Output format\n");
            s.push_str("  HISTSIZE       - Max history entries\n");
            s.push_str("  AUTOCOMMIT     - Auto-commit mode (on/off)\n");
            s
        }
        "if" | "conditional" => {
            let mut s = String::new();
            s.push_str("Conditional Execution\n\n");
            s.push_str("Syntax:\n");
            s.push_str("  \\if <condition>\n");
            s.push_str("    <commands>\n");
            s.push_str("  \\elif <condition>\n");
            s.push_str("    <commands>\n");
            s.push_str("  \\else\n");
            s.push_str("    <commands>\n");
            s.push_str("  \\endif\n\n");
            s.push_str("Conditions:\n");
            s.push_str("  VAR          - True if variable is set\n");
            s.push_str("  ?VAR         - True if variable is set\n");
            s.push_str("  !?VAR        - True if variable is not set\n");
            s.push_str("  VAR == VALUE - True if variable equals value\n");
            s.push_str("  VAR != VALUE - True if variable not equals value\n\n");
            s.push_str("Example:\n");
            s.push_str("  \\set mode test\n");
            s.push_str("  \\if mode == test\n");
            s.push_str("    MATCH (p:person) RETURN p LIMIT 10;\n");
            s.push_str("  \\else\n");
            s.push_str("    MATCH (p:person) RETURN p;\n");
            s.push_str("  \\endif\n");
            s
        }
        "history" => {
            let mut s = String::new();
            s.push_str("Command History\n\n");
            s.push_str("Commands:\n");
            s.push_str("  \\history [N]          - Show last N entries (default 20)\n");
            s.push_str("  \\history search <pat> - Search history for pattern\n");
            s.push_str("  \\history exec <id>    - Re-execute entry by ID\n");
            s.push_str("  \\history clear        - Clear all history\n\n");
            s.push_str("History is saved to ~/.graphdb/cli_history\n");
            s.push_str("Use UP/DOWN arrows to navigate history in the REPL.\n");
            s
        }
        "edit" | "buffer" => {
            let mut s = String::new();
            s.push_str("Query Buffer and External Editor\n\n");
            s.push_str("Commands:\n");
            s.push_str("  \\e [file] [+line]  - Edit in external editor\n");
            s.push_str("  \\p                  - Print current buffer\n");
            s.push_str("  \\r                  - Reset (clear) buffer\n");
            s.push_str("  \\w <file>           - Write buffer to file\n\n");
            s.push_str("The editor is determined by:\n");
            s.push_str("  1. \\set EDITOR <cmd>\n");
            s.push_str("  2. EDITOR environment variable\n");
            s.push_str("  3. VISUAL environment variable\n");
            s.push_str("  4. Default: vi (or notepad on Windows)\n");
            s
        }
        _ => format!(
            "No help available for '{}'. Type \\? for a list of meta-commands.",
            topic
        ),
    }
}

pub fn show_version() -> String {
    format!(
        "GraphDB CLI v{}\nGraphDB - A lightweight single-node graph database",
        env!("CARGO_PKG_VERSION")
    )
}

pub fn show_copyright() -> String {
    "GraphDB CLI\n\
     Copyright (c) 2024 GraphDB Contributors\n\
     Licensed under the Apache License, Version 2.0"
        .to_string()
}
