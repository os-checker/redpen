extern crate rustc_span;

use annotate_snippets::{AnnotationKind, Level, Renderer, Snippet};
use indexmap::IndexMap;
use rustc_middle::ty::TyCtxt;
use rustc_public::{
    CrateDef,
    rustc_internal::internal,
    ty::{FnDef, Span as PubSpan},
};
use rustc_span::{
    Span,
    source_map::{SourceMap, get_source_map},
};
use std::sync::Arc;

pub struct SourceCode<'tcx> {
    tcx: TyCtxt<'tcx>,
    src_map: Arc<SourceMap>,
}

impl SourceCode<'_> {
    pub fn new(tcx: TyCtxt) -> SourceCode {
        SourceCode {
            tcx,
            src_map: get_source_map().unwrap(),
        }
    }
}

struct CheckPanic<'tcx, 'src, 'spots> {
    f: PubSpan,
    spots: &'spots Spots,
    src: &'src SourceCode<'tcx>,
}

impl<'tcx, 'src, 'spots> CheckPanic<'tcx, 'src, 'spots> {
    pub fn new(f: PubSpan, spots: &'spots Spots, src: &'src SourceCode<'tcx>) -> Self {
        CheckPanic { f, spots, src }
    }

    pub fn emit(&self, renderer: &Renderer) {
        let tcx = self.src.tcx;
        let span_func = span(self.spots.caller, tcx);
        let source_map = &self.src.src_map;

        let source = source_map.span_to_snippet(span_func).unwrap_or_else(|err| {
            panic!("Unable to get snippet from this span `{span_func:?}`:\n{err:?}",)
        });

        let pos_func = span_func.lo();
        let loc = source_map.lookup_char_pos(pos_func);
        let file_path = loc.file.name.prefer_remapped_unconditionally().to_string();

        let offset = |sp: PubSpan| {
            let span = span(sp, tcx);
            let call_span_lo = span.lo() - pos_func;
            let call_span_hi = span.hi() - pos_func;
            call_span_lo.0 as usize..call_span_hi.0 as usize
        };

        let annot_caller = AnnotationKind::Context
            .span(offset(self.f))
            .label("For this function.");

        let annot_call = |sp: PubSpan| {
            AnnotationKind::Primary
                .span(offset(sp))
                .label("This may panic!")
        };

        let diag = Level::ERROR
            .primary_title("A possible panic spot is found.")
            .element(
                Snippet::source(source)
                    .path(file_path)
                    .line_start(loc.line)
                    .annotation(annot_caller)
                    .annotations(self.spots.calls.iter().copied().map(annot_call)),
            );
        eprintln!("{}", renderer.render(&[diag]));
    }
}

#[derive(Debug)]
struct Spots {
    caller: PubSpan,
    // FIXME: ensure the spans of calls are within caller span.
    calls: Vec<PubSpan>,
}

fn span(sp: PubSpan, tcx: TyCtxt) -> Span {
    internal(tcx, sp)
}

#[derive(Default, Debug)]
pub struct PanicSpots {
    map: IndexMap<FnDef, Spots>,
}

impl PanicSpots {
    pub fn add(&mut self, caller: FnDef, span: PubSpan, span_callee: PubSpan) {
        self.map
            .entry(caller)
            .and_modify(|v| v.calls.push(span_callee))
            .or_insert_with(|| Spots {
                caller: span,
                calls: vec![span_callee],
            });
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn emit(&self, src: &SourceCode) {
        let renderer = Renderer::styled();
        for (f, calls) in &self.map {
            CheckPanic::new(f.span(), calls, src).emit(&renderer);
        }
    }
}
