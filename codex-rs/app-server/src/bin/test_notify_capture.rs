#[cfg(target_arch = "wasm32")]
fn main() {}

#[cfg(not(target_arch = "wasm32"))]
use anyhow::Result;
#[cfg(not(target_arch = "wasm32"))]
use anyhow::anyhow;
#[cfg(not(target_arch = "wasm32"))]
use std::env;
#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<()> {
    let mut args = env::args_os().skip(1);
    let output_path = PathBuf::from(
        args.next()
            .ok_or_else(|| anyhow!("missing output path argument"))?,
    );
    let payload = args
        .next()
        .ok_or_else(|| anyhow!("missing payload argument"))?
        .into_string()
        .map_err(|_| anyhow!("payload must be valid UTF-8"))?;

    let temp_path = output_path.with_extension("json.tmp");
    std::fs::write(&temp_path, payload)?;
    std::fs::rename(&temp_path, &output_path)?;

    Ok(())
}
