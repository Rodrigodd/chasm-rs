use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn compile(source: &str) -> Result<Vec<u8>, JsValue> {
    chasm_rs::compile(source).map_err(|x| -> JsValue {
        let (line, column) = x.get_line_column();
        let value = &x.source[x.span.clone()];
        let json = format!(r#"{{ "message": {:?}, "token": {{ "value": {:?}, "line": {}, "char": {} }} }}"#, x.to_string(), value, line-1, column-1);
        json.into()
    })
}
