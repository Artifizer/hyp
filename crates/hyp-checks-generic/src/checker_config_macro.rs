//! Macros for defining checker configurations and implementations

/// Define a complete checker with configuration, struct, and trait implementation.
///
/// This macro generates:
/// - The config struct with serde derives and `#[serde(default)]`
/// - The checker struct with config field
/// - A `CONFIG_ENTRY_NAME` constant for registry use
/// - Complete `Checker` trait implementation
/// - Auto-generated methods: `severity()`, `categories()`, `is_enabled()`, `set_config()`
/// - User provides only the `check_item` implementation
///
/// # Example
///
/// ```text
/// define_checker! {
///     /// Checker for E1001: Direct panic calls
///     E1001DirectPanic,
///     code = "E1001",
///     name = "Direct panic() call",
///     suggestions = "Return Result<T, E> instead of panicking",
///     target_items = [Function],
///     config_entry_name = "e1001_direct_panic",
///     /// Configuration for E1001
///     config = E1001Config {
///         /// Whether this checker is enabled
///         enabled: bool = true,
///         /// Severity level
///         severity: crate::config::SeverityLevel = crate::config::SeverityLevel::High,
///         /// Categories
///         categories: Vec<crate::config::CheckerCategory> = vec![...],
///     },
///     check_item(self, item, file_path) {
///         // Your check logic here
///         Ok(violations)
///     }
/// }
/// ```
#[macro_export]
macro_rules! define_checker {
    (
        $(#[$checker_meta:meta])*
        $checker:ident,
        code = $code:expr,
        name = $name:expr,
        suggestions = $suggestions:expr,
        target_items = [$($target:ident),* $(,)?],
        config_entry_name = $config_entry_name:expr,
        $(#[$config_meta:meta])*
        config = $config_name:ident {
            $(
                $(#[$field_meta:meta])*
                $field:ident: $type:ty = $default:expr
            ),* $(,)?
        },
        $(#[$check_item_comment:meta])*
        check_item($self:ident, $item:ident, $file_path:ident) $check_body:block
    ) => {
        // ============================================================
        // Generate config struct
        // ============================================================
        $(#[$config_meta])*
        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
        #[serde(default)]
        #[allow(missing_docs)]
        pub struct $config_name {
            $(
                $(#[$field_meta])*
                pub $field: $type,
            )*
        }

        impl Default for $config_name {
            fn default() -> Self {
                Self {
                    $(
                        $field: $default,
                    )*
                }
            }
        }

        // ============================================================
        // Generate checker struct with CONFIG_ENTRY_NAME constant
        // ============================================================
        $(#[$checker_meta])*
        pub struct $checker {
            config: $config_name,
        }

        impl $checker {
            /// Configuration entry name for registry lookup
            pub const CONFIG_ENTRY_NAME: &'static str = $config_entry_name;
        }

        impl Default for $checker {
            fn default() -> Self {
                Self {
                    config: $config_name::default(),
                }
            }
        }

        impl $checker {
            fn set_config_impl(&mut self, config: Box<dyn std::any::Any>) -> $crate::Result<()> {
                if let Ok(config) = config.downcast::<$config_name>() {
                    self.config = *config;
                    Ok(())
                } else {
                    Err($crate::AnalyzerError::Config(
                        format!("Invalid configuration type for {}", $code),
                    ))
                }
            }
        }

        // ============================================================
        // Implement Checker trait
        // ============================================================
        impl $crate::checker::Checker for $checker {
            fn set_config(&mut self, config: Box<dyn std::any::Any>) -> $crate::Result<()> {
                self.set_config_impl(config)
            }

            fn code(&self) -> &str {
                $code
            }

            fn name(&self) -> &str {
                $name
            }

            fn suggestions(&self) -> &str {
                $suggestions
            }

            fn severity(&self) -> $crate::violation::CheckerSeverity {
                self.config.severity.into()
            }

            fn categories(&self) -> &[$crate::config::CheckerCategory] {
                &self.config.categories
            }

            fn target_items(&self) -> &[$crate::checker::ItemType] {
                &[$($crate::checker::ItemType::$target),*]
            }

            fn check_item(&$self, $item: &syn::Item, $file_path: &str) -> $crate::Result<Vec<$crate::violation::Violation>>
                $check_body

            fn is_enabled(&self) -> bool {
                self.config.enabled
            }
        }
    };
}

/// Register a checker with simplified boilerplate.
///
/// This macro creates a `CheckerRegistration` for a checker type.
///
/// # Example
///
/// ```text
/// use crate::register_checker;
///
/// pub fn e10_registrations() -> Vec<CheckerRegistration> {
///     vec![
///         register_checker!(E1001DirectPanic, E1001Config),
///     ]
/// }
/// ```
#[macro_export]
macro_rules! register_checker {
    ($checker:ty, $config:ty) => {
        $crate::registry::CheckerRegistration {
            descriptor: <$checker>::default().descriptor(),
            factory: |config: &$crate::config::AnalyzerConfig| {
                let cfg: $config = config.get_checker_config(<$checker>::CONFIG_ENTRY_NAME)?;
                if cfg.enabled {
                    let mut checker = <$checker>::default();
                    let _ = checker.set_config(Box::new(cfg));
                    Ok(Some(Box::new(checker)))
                } else {
                    Ok(None)
                }
            },
            config_entry_name: <$checker>::CONFIG_ENTRY_NAME,
        }
    };
}
