use std::env::args;
use std::process::exit;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use anyhow::Result;
use df::dataset::Hdf5Dataset;

fn update_attr(ds: &hdf5::Dataset, name: &str, value: &usize) -> Result<()> {
    let attr = match ds.attr(name) {
        Ok(a) => a,
        Err(_) => ds.new_attr::<usize>().create(name)?,
    };
    attr.write_scalar(value)?;
    Ok(())
}

fn main() -> Result<()> {
    let args = args().collect::<Vec<String>>();
    let p = match args.get(1) {
        Some(p) => p,
        None => {
            eprintln!("HDF5 dataset path expected");
            exit(1);
        }
    };
    let should_stop = Arc::new(AtomicBool::new(false));

    let should_stop_t = should_stop.clone();
    ctrlc::set_handler(move || {
        if should_stop_t.load(Ordering::Acquire) {
            panic!("Stopping");
        }
        println!("Got Ctrl+C! Stopping.");
        should_stop_t.store(true, Ordering::Release);
    })
    .expect("Error setting Ctrl+C handler");

    let ds = Hdf5Dataset::new_rw(p)?;
    for k in ds.keys()? {
        let audio = ds.read_all_channels(&k)?;
        let n_samples = ds.sample_len(&k)?;
        println!(
            "Got sample '{}' with audio shape {:?}, n_samples: {}",
            k,
            audio.shape(),
            n_samples
        );
        let hdf5_ds = ds.ds(&k)?;
        update_attr(&hdf5_ds, "n_samples", &audio.len_of(ndarray::Axis(1)))?;
        update_attr(&hdf5_ds, "n_channels", &audio.len_of(ndarray::Axis(0)))?;
        // dbg!(hdf5_ds.attr("n_samples")?.read_scalar::<usize>()?);
        // dbg!(hdf5_ds.attr("n_channels")?.read_scalar::<usize>()?);
        // break;

        if should_stop.load(Ordering::Acquire) {
            break;
        }
    }
    Ok(())
}
