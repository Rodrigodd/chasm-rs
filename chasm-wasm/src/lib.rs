use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn compile(source: &str) -> Result<Vec<u8>, JsValue> {
    chasm_rs::compile(source).map_err(|x| -> JsValue { x.to_string().into() })
}
