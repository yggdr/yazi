use std::process::Stdio;

use anyhow::Result;
use shared::Url;
use tokio::{io::{AsyncBufReadExt, BufReader}, process::Command, sync::mpsc::{self, UnboundedReceiver}};

use crate::files::File;

pub struct RgOpt {
	pub cwd:     Url,
	pub hidden:  bool,
	pub subject: String,
}

pub fn rg(opt: RgOpt) -> Result<UnboundedReceiver<File>> {
	let mut child = Command::new("rg")
		.current_dir(&opt.cwd)
		.args(["--color=never", "--files-with-matches", "--smart-case"])
		.arg(if opt.hidden { "--hidden" } else { "--no-hidden" })
		.arg(&opt.subject)
		.kill_on_drop(true)
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.spawn()?;

	drop(child.stderr.take());

	let mut it = BufReader::new(child.stdout.take().unwrap()).lines();
	let (tx, rx) = mpsc::unbounded_channel();

	tokio::spawn(async move {
		while let Ok(Some(line)) = it.next_line().await {
			if let Ok(file) = File::from(opt.cwd.join(line)).await {
				tx.send(file).ok();
			}
		}
		child.wait().await.ok();
	});
	Ok(rx)
}
