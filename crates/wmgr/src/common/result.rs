use crate::common::error::WmgrError;

/// wmgrプロジェクト全体で使用するResult型のエイリアス
///
/// このエイリアスにより、プロジェクト全体で一貫したエラーハンドリングが可能になる。
///
/// # Examples
///
/// ```
/// use wmgr::common::result::WmgrResult;
/// use wmgr::common::error::WmgrError;
///
/// fn example_function() -> WmgrResult<String> {
///     Ok("success".to_string())
/// }
///
/// fn example_with_error() -> WmgrResult<()> {
///     Err(WmgrError::internal_error("Something went wrong"))
/// }
/// ```
pub type WmgrResult<T> = Result<T, WmgrError>;

/// Optionのエラー変換ヘルパー
///
/// OptionをWmgrResultに変換するためのヘルパー関数
pub trait OptionExt<T> {
    /// OptionをWmgrResultに変換する
    ///
    /// # Arguments
    ///
    /// * `error` - Noneの場合に返すエラー
    ///
    /// # Examples
    ///
    /// ```
    /// use wmgr::common::result::{WmgrResult, OptionExt};
    /// use wmgr::common::error::WmgrError;
    ///
    /// let some_value: Option<String> = Some("value".to_string());
    /// let result: WmgrResult<String> = some_value.ok_or_wmgr(
    ///     WmgrError::internal_error("Value not found")
    /// );
    /// assert!(result.is_ok());
    ///
    /// let none_value: Option<String> = None;
    /// let result: WmgrResult<String> = none_value.ok_or_wmgr(
    ///     WmgrError::internal_error("Value not found")
    /// );
    /// assert!(result.is_err());
    /// ```
    fn ok_or_wmgr(self, error: WmgrError) -> WmgrResult<T>;

    /// Optionをエラーメッセージ付きでWmgrResultに変換する
    ///
    /// # Arguments
    ///
    /// * `message` - Noneの場合に使用するエラーメッセージ
    ///
    /// # Examples
    ///
    /// ```
    /// use wmgr::common::result::{WmgrResult, OptionExt};
    ///
    /// let none_value: Option<String> = None;
    /// let result: WmgrResult<String> = none_value.ok_or_internal_error("Value not found");
    /// assert!(result.is_err());
    /// ```
    fn ok_or_internal_error(self, message: impl Into<String>) -> WmgrResult<T>;

    /// Option値をValidationErrorに変換する
    ///
    /// # Arguments
    ///
    /// * `field` - バリデーションエラーのフィールド名
    /// * `message` - エラーメッセージ
    ///
    /// # Examples
    ///
    /// ```
    /// use wmgr::common::result::{WmgrResult, OptionExt};
    ///
    /// let none_value: Option<String> = None;
    /// let result: WmgrResult<String> = none_value.ok_or_validation_error("field", "required");
    /// assert!(result.is_err());
    /// ```
    fn ok_or_validation_error(
        self,
        field: impl Into<String>,
        message: impl Into<String>,
    ) -> WmgrResult<T>;
}

impl<T> OptionExt<T> for Option<T> {
    fn ok_or_wmgr(self, error: WmgrError) -> WmgrResult<T> {
        self.ok_or(error)
    }

    fn ok_or_internal_error(self, message: impl Into<String>) -> WmgrResult<T> {
        self.ok_or_else(|| WmgrError::internal_error(message))
    }

    fn ok_or_validation_error(
        self,
        field: impl Into<String>,
        message: impl Into<String>,
    ) -> WmgrResult<T> {
        self.ok_or_else(|| WmgrError::validation_error(field, message, None))
    }
}

/// Resultのエラー変換ヘルパー
///
/// 標準のResult型をWmgrResultに変換するためのヘルパー
pub trait ResultExt<T, E> {
    /// ResultをWmgrResultに変換する
    ///
    /// # Arguments
    ///
    /// * `f` - エラー変換関数
    ///
    /// # Examples
    ///
    /// ```
    /// use wmgr::common::result::{WmgrResult, ResultExt};
    /// use wmgr::common::error::WmgrError;
    /// use std::fs;
    ///
    /// let result: Result<String, std::io::Error> = Ok("content".to_string());
    /// let wmgr_result: WmgrResult<String> = result.map_wmgr_err(|e| {
    ///     WmgrError::filesystem_error_with_source("Failed to read", None, e)
    /// });
    /// assert!(wmgr_result.is_ok());
    /// ```
    fn map_wmgr_err<F>(self, f: F) -> WmgrResult<T>
    where
        F: FnOnce(E) -> WmgrError;

    /// ResultをWmgrResultに変換（エラーメッセージ付き）
    ///
    /// # Arguments
    ///
    /// * `message` - エラーメッセージ
    ///
    /// # Examples
    ///
    /// ```
    /// use wmgr::common::result::{WmgrResult, ResultExt};
    /// use std::fs;
    ///
    /// let result: Result<String, std::io::Error> = Err(std::io::Error::new(
    ///     std::io::ErrorKind::NotFound, "file not found"
    /// ));
    /// let wmgr_result: WmgrResult<String> = result.with_internal_error("File operation failed");
    /// assert!(wmgr_result.is_err());
    /// ```
    fn with_internal_error(self, message: impl Into<String>) -> WmgrResult<T>
    where
        E: std::error::Error + Send + Sync + 'static;

    /// GitエラーとしてWmgrResultに変換
    fn with_git_error(self, message: impl Into<String>) -> WmgrResult<T>
    where
        E: Into<WmgrError>;

    /// ファイルシステムエラーとしてWmgrResultに変換
    fn with_filesystem_error(
        self,
        message: impl Into<String>,
        path: Option<std::path::PathBuf>,
    ) -> WmgrResult<T>
    where
        E: Into<std::io::Error>;
}

impl<T, E> ResultExt<T, E> for Result<T, E> {
    fn map_wmgr_err<F>(self, f: F) -> WmgrResult<T>
    where
        F: FnOnce(E) -> WmgrError,
    {
        self.map_err(f)
    }

    fn with_internal_error(self, message: impl Into<String>) -> WmgrResult<T>
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        self.map_err(|e| WmgrError::internal_error_with_source(message, e))
    }

    fn with_git_error(self, message: impl Into<String>) -> WmgrResult<T>
    where
        E: Into<WmgrError>,
    {
        self.map_err(|e| {
            let wmgr_error = e.into();
            match wmgr_error {
                WmgrError::GitError { .. } => wmgr_error,
                _ => WmgrError::git_error(message),
            }
        })
    }

    fn with_filesystem_error(
        self,
        message: impl Into<String>,
        path: Option<std::path::PathBuf>,
    ) -> WmgrResult<T>
    where
        E: Into<std::io::Error>,
    {
        self.map_err(|e| {
            let io_error = e.into();
            WmgrError::filesystem_error_with_source(message, path, io_error)
        })
    }
}

/// チェーンオペレーション用のヘルパー
///
/// 複数のWmgrResult操作を連鎖させるためのヘルパー
pub trait WmgrResultExt<T> {
    /// エラー時にコンテキストを追加
    ///
    /// # Arguments
    ///
    /// * `context` - 追加するコンテキスト
    ///
    /// # Examples
    ///
    /// ```
    /// use wmgr::common::result::{WmgrResult, WmgrResultExt};
    /// use wmgr::common::error::WmgrError;
    ///
    /// let result: WmgrResult<String> = Err(WmgrError::internal_error("original error"));
    /// let with_context = result.with_context("operation failed");
    /// assert!(with_context.is_err());
    /// ```
    fn with_context(self, context: impl Into<String>) -> WmgrResult<T>;

    /// Optionに変換（エラーをログ出力）
    fn to_option_logged(self) -> Option<T>;

    /// デフォルト値でエラーを無視
    fn unwrap_or_default_logged(self) -> T
    where
        T: Default;
}

impl<T> WmgrResultExt<T> for WmgrResult<T> {
    fn with_context(self, context: impl Into<String>) -> WmgrResult<T> {
        self.map_err(|e| WmgrError::internal_error_with_source(context, e))
    }

    fn to_option_logged(self) -> Option<T> {
        match self {
            Ok(value) => Some(value),
            Err(e) => {
                tracing::error!("WmgrResult error: {}", e);
                None
            }
        }
    }

    fn unwrap_or_default_logged(self) -> T
    where
        T: Default,
    {
        match self {
            Ok(value) => value,
            Err(e) => {
                tracing::error!("WmgrResult error, using default: {}", e);
                T::default()
            }
        }
    }
}

/// async関数用のヘルパー
pub mod async_helpers {
    use super::{WmgrError, WmgrResult};
    use std::future::Future;

    /// タイムアウト付きasync実行
    pub async fn with_timeout<F, T>(f: F, timeout_secs: u64) -> WmgrResult<T>
    where
        F: Future<Output = WmgrResult<T>>,
    {
        let timeout_duration = std::time::Duration::from_secs(timeout_secs);

        match tokio::time::timeout(timeout_duration, f).await {
            Ok(result) => result,
            Err(_) => Err(WmgrError::timeout(timeout_secs)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_option_ext_ok_or_wmgr() {
        let some_value = Some("test".to_string());
        let result = some_value.ok_or_wmgr(WmgrError::internal_error("error"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test");

        let none_value: Option<String> = None;
        let result = none_value.ok_or_wmgr(WmgrError::internal_error("error"));
        assert!(result.is_err());
    }

    #[test]
    fn test_option_ext_ok_or_internal_error() {
        let none_value: Option<String> = None;
        let result = none_value.ok_or_internal_error("test error");
        assert!(result.is_err());

        if let Err(WmgrError::InternalError { message, .. }) = result {
            assert_eq!(message, "test error");
        } else {
            panic!("Expected InternalError");
        }
    }

    #[test]
    fn test_option_ext_ok_or_validation_error() {
        let none_value: Option<String> = None;
        let result = none_value.ok_or_validation_error("field", "required");
        assert!(result.is_err());

        if let Err(WmgrError::ValidationError { field, message, .. }) = result {
            assert_eq!(field, "field");
            assert_eq!(message, "required");
        } else {
            panic!("Expected ValidationError");
        }
    }

    #[test]
    fn test_result_ext_map_wmgr_err() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let result: Result<String, std::io::Error> = Err(io_error);

        let wmgr_result =
            result.map_wmgr_err(|e| WmgrError::filesystem_error_with_source("test", None, e));

        assert!(wmgr_result.is_err());
    }

    #[test]
    fn test_result_ext_with_filesystem_error() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let result: Result<String, std::io::Error> = Err(io_error);
        let path = Some(PathBuf::from("/test/path"));

        let wmgr_result = result.with_filesystem_error("test operation", path);
        assert!(wmgr_result.is_err());
    }

    #[test]
    fn test_wmgr_result_ext_with_context() {
        let result: WmgrResult<String> = Err(WmgrError::internal_error("original"));
        let with_context = result.with_context("additional context");

        assert!(with_context.is_err());
    }

    #[test]
    fn test_wmgr_result_ext_to_option_logged() {
        let ok_result: WmgrResult<String> = Ok("test".to_string());
        assert_eq!(ok_result.to_option_logged(), Some("test".to_string()));

        let err_result: WmgrResult<String> = Err(WmgrError::internal_error("error"));
        assert_eq!(err_result.to_option_logged(), None);
    }

    #[test]
    fn test_wmgr_result_ext_unwrap_or_default_logged() {
        let ok_result: WmgrResult<String> = Ok("test".to_string());
        assert_eq!(ok_result.unwrap_or_default_logged(), "test");

        let err_result: WmgrResult<String> = Err(WmgrError::internal_error("error"));
        assert_eq!(err_result.unwrap_or_default_logged(), String::default());
    }

    #[tokio::test]
    async fn test_async_helpers_with_timeout() {
        use super::async_helpers::*;

        // 成功ケース
        let fast_future = async { Ok("result".to_string()) };
        let result = with_timeout(fast_future, 1).await;
        assert!(result.is_ok());

        // タイムアウトケース
        let slow_future = async {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            Ok("result".to_string())
        };
        let result = with_timeout(slow_future, 1).await;
        assert!(result.is_err());

        if let Err(WmgrError::Timeout { timeout_secs }) = result {
            assert_eq!(timeout_secs, 1);
        } else {
            panic!("Expected Timeout error");
        }
    }
}
