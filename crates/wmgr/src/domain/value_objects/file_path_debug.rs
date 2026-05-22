// Debug用: FilePathでdotエラーのテスト

#[cfg(test)]
mod debug_tests {
    use super::super::file_path::{FilePath, FilePathError};

    #[test]
    fn test_dot_pattern_detection() {
        // テストケース1: 単体の "."
        println!("Testing single dot '.'");
        let result = FilePath::new(".");
        match result {
            Ok(fp) => println!("✓ Success: {}", fp.as_str()),
            Err(e) => println!("✗ Error: {}", e),
        }

        // テストケース2: "./"を含むパス
        println!("Testing path with './'");
        let result = FilePath::new("./somedir");
        match result {
            Ok(fp) => println!("✓ Success: {}", fp.as_str()),
            Err(e) => println!("✗ Error: {}", e),
        }

        // テストケース3: ファイル名に.を含むパス（許可されるべき）
        println!("Testing file with extension");
        let result = FilePath::new("file.txt");
        match result {
            Ok(fp) => println!("✓ Success: {}", fp.as_str()),
            Err(e) => println!("✗ Error: {}", e),
        }

        // テストケース4: 正規化後に"."になるパターン
        println!("Testing normalized dot pattern");
        let current_dir = std::env::current_dir().unwrap();
        let repo_path = current_dir.join(".");
        let path_str = repo_path.to_string_lossy();
        println!("Full path: {}", path_str);
        let result = FilePath::new(&path_str);
        match result {
            Ok(fp) => println!("✓ Success: {}", fp.as_str()),
            Err(e) => println!("✗ Error: {}", e),
        }
    }

    #[test]
    fn test_dangerous_patterns_directly() {
        let dangerous_patterns = [
            "|",
            "&",
            ";",
            "$",
            "`",
            "$(",
            "${",
            "<script",
            "javascript:",
            "data:",
            "%2e%2e",
            "%2e%2e%2f",
            "%2e%2e%5c",
            "..%2f",
            "..%5c",
            "%252e%252e",
            "\u{2024}",
            "\u{ff0e}",
            "%252e",
            "%255c",
            "%252f",
        ];

        for pattern in &dangerous_patterns {
            let test_path = format!("file{}", pattern);
            let result = FilePath::new(&test_path);
            println!(
                "Testing pattern '{}' in path '{}': {:?}",
                pattern, test_path, result
            );
            if let Err(FilePathError::DangerousPattern(detected)) = result {
                println!("Detected dangerous pattern: {}", detected);
            }
        }
    }
}
