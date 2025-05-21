#![cfg(feature = "font_ubuntu_light_compressed")]

use js_sys::{Array, Uint8Array};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Blob, CompressionFormat, ReadableWritablePair, Response};

pub async fn decompress_gzip(compressed_data: &[u8]) -> Result<Vec<u8>, JsValue> {
    // 1. Create a Blob from the compressed data
    let js_array = Uint8Array::from(compressed_data);
    let array = Array::new();
    array.push(&js_array);

    // Create a Blob containing the compressed data
    let options = web_sys::BlobPropertyBag::new();
    let blob = Blob::new_with_u8_array_sequence_and_options(&array, &options)?;

    // 2. Get a ReadableStream from the Blob
    let stream = blob.stream();

    // 3. Create a DecompressionStream and pipe through it
    // Cast the DecompressionStream to ReadableWritablePair which is what pipe_through expects
    let decompressor = web_sys::DecompressionStream::new(CompressionFormat::Gzip)?;
    let transform_stream: &ReadableWritablePair = decompressor.unchecked_ref();
    let decompressed_stream = stream.pipe_through(transform_stream);

    // 4. Create a Response from the decompressed stream to access its buffer
    let response = Response::new_with_opt_readable_stream(Some(&decompressed_stream))?;

    // 5. Get the full content as an ArrayBuffer
    let buffer_promise = response.array_buffer()?;
    let buffer = JsFuture::from(buffer_promise).await?;

    // 6. Convert to Rust Vec<u8>
    let result_array = Uint8Array::new(&buffer);
    let mut result = vec![0; result_array.length() as usize];
    result_array.copy_to(&mut result);

    Ok(result)
}
