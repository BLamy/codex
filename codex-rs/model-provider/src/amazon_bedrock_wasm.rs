/// The browser build does not include the AWS SDK or Bedrock credential chain.
pub fn is_supported_amazon_bedrock_region(_region: &str) -> bool {
    false
}
