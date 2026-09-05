//! Uid → login-name resolution for the processes view (UX9.1).
//!
//! `ProcessInfo::user_id` carries the numeric uid; the processes widget wants
//! to show the real user name. That mapping is a *display* concern, so it
//! lives here (kernel state), not in the data model. On unix platforms the
//! names come from `/etc/passwd`, parsed once at startup into a
//! `HashMap<u32, String>`; on platforms without `/etc/passwd` the map stays
//! empty and renderers fall back to the numeric uid (the
//! `WidgetState::uid_to_name` contract default is the same `None` fallback).
//!
//! The parser accepts the standard 7-field `passwd(5)` format and skips
//! anything malformed: blank lines, `#` comments and entries without a valid
//! numeric uid or a user name are ignored. When the same uid appears more
//! than once, the last entry wins.

use std::collections::HashMap;
use std::fs;

/// Login names keyed by numeric uid.
#[derive(Debug, Default, Clone)]
pub struct Users {
    by_uid: HashMap<u32, String>,
}

impl Users {
    /// Load the user table from `/etc/passwd` (unix). On platforms where the
    /// file does not exist the read fails and the table stays empty — the
    /// numeric-uid fallback then applies everywhere.
    pub fn load() -> Self {
        match fs::read_to_string("/etc/passwd") {
            Ok(contents) => Self::parse(&contents),
            Err(_) => Self::default(),
        }
    }

    /// Parse `passwd(5)` text into a uid → name table (pure, testable).
    fn parse(contents: &str) -> Self {
        let mut by_uid = HashMap::new();
        for line in contents.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let mut fields = line.splitn(7, ':');
            let (Some(name), Some(_password), Some(uid), Some(_gid)) =
                (fields.next(), fields.next(), fields.next(), fields.next())
            else {
                continue;
            };
            if name.is_empty() {
                continue;
            }
            let Ok(uid) = uid.parse::<u32>() else {
                continue;
            };
            by_uid.insert(uid, name.to_string());
        }
        Self { by_uid }
    }

    /// The login name for a uid, or `None` when unknown (numeric fallback).
    pub fn name_for(&self, uid: u32) -> Option<&str> {
        self.by_uid.get(&uid).map(String::as_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_reads_standard_passwd_lines() {
        let users = Users::parse(
            "root:x:0:0:root:/root:/bin/bash\n\
             daemon:x:1:1:daemon:/usr/sbin:/usr/sbin/nologin\n\
             xscriptor:x:1000:1000:X Scriptor:/home/xscriptor:/bin/zsh\n",
        );
        assert_eq!(users.name_for(0), Some("root"));
        assert_eq!(users.name_for(1), Some("daemon"));
        assert_eq!(users.name_for(1000), Some("xscriptor"));
        assert_eq!(users.name_for(2), None, "unknown uid falls back to numeric");
    }

    #[test]
    fn parse_skips_comments_blanks_and_malformed_lines() {
        let users = Users::parse(
            "# comment line\n\
             \n\
             broken-line-without-colons\n\
             user::only-three:fields\n\
             baduid:x:notanumber:0::/tmp:/bin/false\n\
             nopasswd:x::0::/tmp:/bin/false\n\
             :x:0:0::/tmp:/bin/false\n",
        );
        assert!(users.name_for(0).is_none());
        assert!(users.name_for(3).is_none());
    }

    #[test]
    fn last_entry_wins_on_duplicate_uid() {
        let users = Users::parse("a:x:10:0::/tmp:/bin/false\nb:x:10:0::/tmp:/bin/false\n");
        assert_eq!(users.name_for(10), Some("b"));
    }

    #[test]
    fn empty_input_yields_empty_table() {
        let users = Users::parse("");
        assert_eq!(users.name_for(0), None);
        assert!(Users::default().name_for(1000).is_none());
    }
}
