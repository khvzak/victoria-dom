
pub fn xml_escape(_text: &String) -> String {
    let mut text = _text.clone();
    text = text.replace("&", "&amp;");
    text = text.replace("<", "&lt;");
    text = text.replace(">", "&gt;");
    text = text.replace("\"", "&quot;");
    text = text.replace("'", "&#39;");
    text
}

pub fn xml_unescape(_text: &String) -> String {
    let mut text = _text.clone();
    text = text.replace("&#39;", "'");
    text = text.replace("&quot;", "\"");
    text = text.replace("&lt;", "<");
    text = text.replace("&gt;", ">");
    text = text.replace("&amp;", "&");
    text
}
