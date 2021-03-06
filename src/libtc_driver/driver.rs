use config::{self, Input};
use metadata::cstore::CStore;
use session::{Session, CompileResult};
use util::common::time;



use syntax::ast;
use syntax::parse::{self, PResult, token};

use super::Compilation;

pub fn compile_input(sess: &Session,
                     cstore: &CStore,
                     cfg: ast::CrateConfig,
                     input: &Input,
                     addl_plugins: Option<Vec<String>>,
                     control: &CompileController) -> CompileResult {

  macro_rules! controller_entry_point {
      ($point: ident, $tsess: expr, $make_state: expr, $phase_result: expr) => {{
          let state = $make_state;
          let phase_result: &CompileResult = &$phase_result;
          if phase_result.is_ok() || control.$point.run_callback_on_error {
              (control.$point.callback)(state);
          }

          if control.$point.stop == Compilation::Stop {
              return compile_result_from_err_count($tsess.err_count());
          }
      }}
  }
  unimplemented!()
}

/// The name used for source code that doesn't originate in a file
/// (e.g. source from stdin or a string)
pub fn anon_src() -> String {
    "<anon>".to_string()
}

/// CompileController is used to customise compilation, it allows compilation to
/// be stopped and/or to call arbitrary code at various points in compilation.
/// It also allows for various flags to be set to influence what information gets
/// collected during compilation.
///
/// This is a somewhat higher level controller than a Session - the Session
/// controls what happens in each phase, whereas the CompileController controls
/// whether a phase is run at all and whether other code (from outside the
/// the compiler) is run between phases.
///
/// Note that if compilation is set to stop and a callback is provided for a
/// given entry point, the callback is called before compilation is stopped.
///
/// Expect more entry points to be added in the future.
pub struct CompileController<'a> {
  pub after_parse: PhaseController<'a>,
  pub after_expand: PhaseController<'a>,
  pub after_write_deps: PhaseController<'a>,
  pub after_analysis: PhaseController<'a>,
  pub after_llvm: PhaseController<'a>,
}

impl<'a> CompileController<'a> {
    pub fn basic() -> CompileController<'a> {
        CompileController {
            after_parse: PhaseController::basic(),
            after_expand: PhaseController::basic(),
            after_write_deps: PhaseController::basic(),
            after_analysis: PhaseController::basic(),
            after_llvm: PhaseController::basic(),
        }
    }
}

pub struct PhaseController<'a> {
    pub stop: Compilation,
    // If true then the compiler will try to run the callback even if the phase
    // ends with an error. Note that this is not always possible.
    pub run_callback_on_error: bool,
    pub callback: Box<Fn(CompileState) -> () + 'a>,
}

impl<'a> PhaseController<'a> {
    pub fn basic() -> PhaseController<'a> {
        PhaseController {
            stop: Compilation::Continue,
            run_callback_on_error: false,
            callback: Box::new(|_| {}),
        }
    }
}

/// State that is passed to a callback. What state is available depends on when
/// during compilation the callback is made. See the various constructor methods
/// (`state_*`) in the impl to see which data is provided for any given entry point.
pub struct CompileState;

pub fn phase_1_parse_input<'a>(sess: &'a Session,
                               cfg: ast::CrateConfig,
                               input: &Input)
                               -> PResult<'a, ast::Crate> {

 let krate = time(sess.time_passes(), "parsing", || {
    match *input {
      Input::File(ref file) => {
        parse::parse_crate_from_file(file, cfg.clone(), &sess.parse_sess)
      }
      Input::Str { ref input, ref name } => {
        parse::parse_crate_from_source_str(name.clone(),
                                            input.clone(),
                                            cfg.clone(),
                                            &sess.parse_sess)
      }
    }
 })?;

 unimplemented!()
}