#![feature(let_chains)]

mod compiler;
mod options;
mod plugin;
mod transformer;

use std::default::Default;

use compiler::{IntoJsAst, SwcCompiler};
use options::SwcCompilerOptionsWithAdditional;
pub use options::SwcLoaderJsOptions;
pub use plugin::SwcLoaderPlugin;
use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{Mode, RunnerContext};
use rspack_error::{AnyhowResultToRspackResultExt, Diagnostic, Result};
use rspack_loader_runner::{Identifiable, Identifier, Loader, LoaderContext};
use rspack_plugin_javascript::{
  ast::{self, SourceMapConfig},
  TransformOutput,
};
use rspack_util::source_map::SourceMapKind;
use swc_config::{config_types::MergingOption, merge::Merge};
use swc_core::{
  base::config::{InputSourceMap, OutputCharset, SourceMapsConfig, TransformConfig},
  ecma::visit::VisitWith,
};
use transformer::IdentCollector;

#[cacheable]
#[derive(Debug)]
pub struct SwcLoader {
  identifier: Identifier,
  options_with_additional: SwcCompilerOptionsWithAdditional,
}

impl SwcLoader {
  pub fn new(raw_options: &str) -> Result<Self, serde_json::Error> {
    Ok(Self {
      identifier: SWC_LOADER_IDENTIFIER.into(),
      options_with_additional: raw_options.try_into()?,
    })
  }

  /// Panics:
  /// Panics if `identifier` passed in is not starting with `builtin:swc-loader`.
  pub fn with_identifier(mut self, identifier: Identifier) -> Self {
    assert!(identifier.starts_with(SWC_LOADER_IDENTIFIER));
    self.identifier = identifier;
    self
  }

  fn loader_impl(&self, loader_context: &mut LoaderContext<RunnerContext>) -> Result<()> {
    let resource_path = loader_context
      .resource_path()
      .map(|p| p.to_path_buf())
      .unwrap_or_default();
    let Some(content) = loader_context.take_content() else {
      return Ok(());
    };

    let swc_options = {
      let mut swc_options = self.options_with_additional.swc_options.clone();
      if swc_options.config.jsc.transform.as_ref().is_some() {
        let mut transform = TransformConfig::default();
        transform.react.development =
          Some(Mode::is_development(&loader_context.context.options.mode));
        swc_options
          .config
          .jsc
          .transform
          .merge(MergingOption::from(Some(transform)));
      }
      if let Some(pre_source_map) = loader_context.source_map().cloned() {
        if let Ok(source_map) = pre_source_map.to_json() {
          swc_options.config.input_source_map = Some(InputSourceMap::Str(source_map))
        }
      }
      swc_options.filename = resource_path.as_str().to_string();
      swc_options.source_file_name = Some(resource_path.as_str().to_string());

      if swc_options.config.jsc.target.is_some() && swc_options.config.env.is_some() {
        loader_context.emit_diagnostic(Diagnostic::warn(
          SWC_LOADER_IDENTIFIER.to_string(),
          "`env` and `jsc.target` cannot be used together".to_string(),
        ));
      }
      swc_options
    };

    let source_map_kind: SourceMapKind = match swc_options.config.source_maps {
      Some(SourceMapsConfig::Bool(false)) => SourceMapKind::empty(),
      _ => loader_context.context.module_source_map_kind,
    };

    let source = content.into_string_lossy();
    let c = SwcCompiler::new(resource_path.into_std_path_buf(), source, swc_options)
      .to_rspack_result_from_anyhow()?;

    let built = c
      .parse(None, |_| {
        transformer::transform(&self.options_with_additional.rspack_experiments)
      })
      .to_rspack_result_from_anyhow()?;

    let input_source_map = c
      .input_source_map(&built.input_source_map)
      .to_rspack_result_from_anyhow()?;
    let mut codegen_options = ast::CodegenOptions {
      target: Some(built.target),
      minify: Some(built.minify),
      emit_assert_for_import_attributes: Some(built.emit_assert_for_import_attributes),
      input_source_map: input_source_map.as_ref(),
      ascii_only: built
        .output
        .charset
        .as_ref()
        .map(|v| matches!(v, OutputCharset::Ascii)),
      source_map_config: SourceMapConfig {
        enable: source_map_kind.source_map(),
        inline_sources_content: source_map_kind.source_map(),
        emit_columns: !source_map_kind.cheap(),
        names: Default::default(),
      },
      inline_script: Some(false),
      keep_comments: Some(true),
    };

    let program = c.transform(built)?;
    if source_map_kind.enabled() {
      let mut v = IdentCollector {
        names: Default::default(),
      };
      program.visit_with(&mut v);
      codegen_options.source_map_config.names = v.names;
    }
    let ast = c.into_js_ast(program);
    let TransformOutput { code, map } = ast::stringify(&ast, codegen_options)?;
    loader_context.finish_with((code, map));

    Ok(())
  }
}

pub const SWC_LOADER_IDENTIFIER: &str = "builtin:swc-loader";

#[cacheable_dyn]
#[async_trait::async_trait]
impl Loader<RunnerContext> for SwcLoader {
  #[tracing::instrument("loader:builtin-swc", skip_all, fields(
    id2 =loader_context.resource(),
  ))]
  async fn run(&self, loader_context: &mut LoaderContext<RunnerContext>) -> Result<()> {
    #[allow(unused_mut)]
    let mut inner = || self.loader_impl(loader_context);
    #[cfg(all(debug_assertions, not(target_family = "wasm")))]
    {
      // Adjust stack to avoid stack overflow.
      stacker::maybe_grow(
        2 * 1024 * 1024, /* 2mb */
        4 * 1024 * 1024, /* 4mb */
        inner,
      )
    }
    #[cfg(any(not(debug_assertions), target_family = "wasm"))]
    inner()
  }
}

impl Identifiable for SwcLoader {
  fn identifier(&self) -> Identifier {
    self.identifier
  }
}
