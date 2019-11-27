use clap::{App, AppSettings, Arg};
use conllx::io::{ReadSentence, WriteSentence};
use std::io::BufWriter;
use stdinout::{Input, OrExit, Output};
use lumberjack;
use lumberjack::io::TryFromConllx;
use lumberjack::{Features, Tree};
use std::ops::{IndexMut};
use petgraph::prelude::NodeIndex;

static INPUT: &str = "INPUT";
static DEFAULT_CLAP_SETTINGS: &[AppSettings] = &[
    AppSettings::DontCollapseArgsInUsage,
    AppSettings::UnifiedHelpMessage,
];
static OUTPUT: &str = "OUTPUT";

fn main() {
    let matches = App::new("depth")
        .settings(DEFAULT_CLAP_SETTINGS)
        .arg(Arg::with_name(INPUT).help("Input").index(1))
        .arg(Arg::with_name(OUTPUT).help("Output").index(2))
        .get_matches();
    let input = matches.value_of(INPUT).map(ToOwned::to_owned);
    let input = Input::from(input);
    let input = conllx::io::Reader::new(input.buf_read().or_exit("Failed opening input", 1));
    let output = matches.value_of(OUTPUT).map(ToOwned::to_owned);
    let output = Output::from(output);
    let output = output.write().or_exit("Failed opening output", 1);

    let mut output = conllx::io::Writer::new(BufWriter::new(output));

    for sent in input.sentences() {
        let sent = sent.unwrap();
        let mut tree = lumberjack::Tree::try_from_conllx_with_relative_encoding(&sent).expect("tree fail");
        let terminals = tree.terminals().collect::<Vec<NodeIndex<u32>>>();
        for terminal in terminals {
            let count = tree.path_iter(terminal).count();
            let node = tree.index_mut(terminal);
            let feature = node.features().unwrap().to_string() + &format!("|cdepth:{}",count);
            node.set_features(Some(Features::from(feature)));
        }
        let out_sent = tree.into();
//        let clone = sent.clone();
//
//        let g_no_mut = clone.dep_graph();
//        let mut g = sent.dep_graph_mut();
//
//        for idx in 1..g.len() {
//            let depth = g_no_mut.path_iter(idx).count();
//            let feats = g[idx]
//                .token()
//                .or_exit("Token missing", 1)
//                .features()
//                .or_exit("Feats missing", 1)
//                .as_str()
//                .to_string()
//                + &format!("|depth:{}", depth);
//            g[idx]
//                .token_mut()
//                .or_exit("Token missing", 1)
//                .set_features(Some(Features::from_string(feats)));
//        }
        output
            .write_sentence(&out_sent)
            .or_exit("Failed writing sent.", 1);
    }
}

/// Trait to provide iterators over the path in a tree from `start` to the root.
///
/// Taken from finalfrontier deps.rs
pub trait PathIter {
    fn path_iter(&self, start: NodeIndex<u32>) -> PathIterator;
}

impl<'a> PathIter for Tree {
    fn path_iter(&self, start: NodeIndex<u32>) -> PathIterator {
        PathIterator {
            graph: self,
            current: start,
        }
    }
}

/// Iterator over the path from the given start node to the root node.
///
/// The path does not include the start node itself.
///
/// Taken from finalfrontier deps.rs
pub struct PathIterator<'a> {
    current: NodeIndex<u32>,
    graph: &'a Tree,
}

impl Iterator for PathIterator<'_> {
    type Item = ();

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((index,_)) = self.graph.parent(self.current) {
            self.current = index;
            Some(())
        } else {
            None
        }
    }
}
