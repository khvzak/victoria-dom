
pub fn xml_escape(text: &str) -> String {
    let mut text = text.to_string();
    text = text.replace("&", "&amp;");
    text = text.replace("<", "&lt;");
    text = text.replace(">", "&gt;");
    text = text.replace("\"", "&quot;");
    text = text.replace("'", "&#39;");
    text
}

pub fn xml_unescape(text: &str) -> String {
    let mut text = text.to_string();
    text = text.replace("&#39;", "'");
    text = text.replace("&quot;", "\"");
    text = text.replace("&lt;", "<");
    text = text.replace("&gt;", ">");
    text = text.replace("&amp;", "&");
    text
}
