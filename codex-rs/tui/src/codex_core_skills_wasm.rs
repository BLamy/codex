pub mod model {
    use codex_protocol::protocol::Product;
    use codex_protocol::protocol::SkillScope;
    use codex_utils_absolute_path::AbsolutePathBuf;

    #[derive(Debug, Clone, PartialEq)]
    pub struct SkillMetadata {
        pub name: String,
        pub description: String,
        pub short_description: Option<String>,
        pub interface: Option<SkillInterface>,
        pub dependencies: Option<SkillDependencies>,
        pub policy: Option<SkillPolicy>,
        pub path_to_skills_md: AbsolutePathBuf,
        pub scope: SkillScope,
        pub plugin_id: Option<String>,
    }

    impl SkillMetadata {
        pub fn allows_implicit_invocation(&self) -> bool {
            self.policy
                .as_ref()
                .and_then(|policy| policy.allow_implicit_invocation)
                .unwrap_or(true)
        }

        pub fn matches_product_restriction_for_product(
            &self,
            restriction_product: Option<Product>,
        ) -> bool {
            match &self.policy {
                Some(policy) => {
                    policy.products.is_empty()
                        || restriction_product.is_some_and(|product| {
                            product.matches_product_restriction(&policy.products)
                        })
                }
                None => true,
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Default)]
    pub struct SkillPolicy {
        pub allow_implicit_invocation: Option<bool>,
        pub products: Vec<Product>,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct SkillInterface {
        pub display_name: Option<String>,
        pub short_description: Option<String>,
        pub icon_small: Option<AbsolutePathBuf>,
        pub icon_large: Option<AbsolutePathBuf>,
        pub brand_color: Option<String>,
        pub default_prompt: Option<String>,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct SkillDependencies {
        pub tools: Vec<SkillToolDependency>,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct SkillToolDependency {
        pub r#type: String,
        pub value: String,
        pub description: Option<String>,
        pub transport: Option<String>,
        pub command: Option<String>,
        pub url: Option<String>,
    }
}
