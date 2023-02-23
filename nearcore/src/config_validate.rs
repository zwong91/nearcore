use near_config_utils::{ValidationError, ValidationErrors};

use crate::config::Config;

/// Validate Config extracted from config.json.
/// This function does not panic. It returns the error if any validation fails.
pub fn validate_config(config: &Config) -> Result<(), ValidationError> {
    let mut validation_errors = ValidationErrors::new();
    let mut config_validator = ConfigValidator::new(config, &mut validation_errors);
    tracing::info!(target: "config", "Validating Config, extracted from config.json...");
    config_validator.validate()
}

struct ConfigValidator<'a> {
    config: &'a Config,
    validation_errors: &'a mut ValidationErrors,
}

impl<'a> ConfigValidator<'a> {
    fn new(config: &'a Config, validation_errors: &'a mut ValidationErrors) -> Self {
        Self { config, validation_errors }
    }

    fn validate(&mut self) -> Result<(), ValidationError> {
        self.validate_all_conditions();
        self.result_with_full_error()
    }

    /// this function would check all conditions, and add all error messages to ConfigValidator.errors
    fn validate_all_conditions(&mut self) {
        if self.config.archive == false && self.config.save_trie_changes == Some(false) {
            let error_message = format!("Configuration with archive = false and save_trie_changes = false is not supported because non-archival nodes must save trie changes in order to do do garbage collection.");
            self.validation_errors.push_config_semantics_error(error_message)
        }

        if self.config.consensus.min_block_production_delay
            > self.config.consensus.max_block_production_delay
        {
            let error_message = format!(
                "min_block_production_delay: {:?} is greater than max_block_production_delay: {:?}",
                self.config.consensus.min_block_production_delay,
                self.config.consensus.max_block_production_delay
            );
            self.validation_errors.push_config_semantics_error(error_message)
        }

        if self.config.consensus.min_block_production_delay
            > self.config.consensus.max_block_wait_delay
        {
            let error_message = format!(
                "min_block_production_delay: {:?} is greater than max_block_wait_delay: {:?}",
                self.config.consensus.min_block_production_delay,
                self.config.consensus.max_block_wait_delay
            );
            self.validation_errors.push_config_semantics_error(error_message)
        }

        if self.config.consensus.header_sync_expected_height_per_second == 0 {
            let error_message =
                format!("consensus.header_sync_expected_height_per_second should not be 0");
            self.validation_errors.push_config_semantics_error(error_message)
        }

        if self.config.gc.gc_blocks_limit == 0
            || self.config.gc.gc_fork_clean_step == 0
            || self.config.gc.gc_num_epochs_to_keep == 0
        {
            let error_message = format!("gc config values should all be greater than 0, but gc_blocks_limit is {:?}, gc_fork_clean_step is {}, gc_num_epochs_to_keep is {}.", self.config.gc.gc_blocks_limit, self.config.gc.gc_fork_clean_step, self.config.gc.gc_num_epochs_to_keep);
            self.validation_errors.push_config_semantics_error(error_message)
        }
    }

    fn result_with_full_error(&self) -> Result<(), ValidationError> {
        if self.validation_errors.is_empty() {
            Ok(())
        } else {
            let full_err_msg = self.validation_errors.generate_error_message_per_type().unwrap();
            Err(ValidationError::ConfigSemanticsError { error_message: full_err_msg }.into())
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[should_panic(expected = "gc config values should all be greater than 0")]
    fn test_gc_config_value_nonzero() {
        let mut config = Config::default();
        config.gc.gc_blocks_limit = 0;
        // set tracked_shards to be non-empty
        config.tracked_shards.push(20);
        validate_config(&config).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "Configuration with archive = false and save_trie_changes = false is not supported"
    )]
    fn test_archive_false_save_trie_changes_false() {
        let mut config = Config::default();
        config.archive = false;
        config.save_trie_changes = Some(false);
        // set tracked_shards to be non-empty
        config.tracked_shards.push(20);
        validate_config(&config).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "\\nconfig.json semantic issue: Configuration with archive = false and save_trie_changes = false is not supported because non-archival nodes must save trie changes in order to do do garbage collection.\\nconfig.json semantic issue: gc config values should all be greater than 0"
    )]
    fn test_multiple_config_validation_errors() {
        let mut config = Config::default();
        config.archive = false;
        config.save_trie_changes = Some(false);
        config.gc.gc_blocks_limit = 0;
        // set tracked_shards to be non-empty
        config.tracked_shards.push(20);
        validate_config(&config).unwrap();
    }
}