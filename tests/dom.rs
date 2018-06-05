extern crate victoria_dom;

use victoria_dom::DOM;

#[test]
fn empty_vals() {
    assert_eq!(DOM::new("").to_string(), "");
    assert_eq!(DOM::new("").content(), "");
    assert!(DOM::new("").at("p").is_none());
    assert_eq!(DOM::new("").text(), "");
}

#[test]
fn nodestroy() {
    // fix issue #4
    let dom = DOM::new("<html>").childs(None);
    assert_eq!(dom[0].text(), "");
}

#[test]
fn basic1() {
    // Simple (basics)
    let dom = DOM::new(r#"<div><div FOO="0" id="a">A</div><div id="b" myAttr>B</div></div>"#);
    assert_eq!(dom.at("#b").unwrap().text(), "B");
    assert_eq!(dom.find("div[id]").iter().map(|x| x.text()).collect::<Vec<_>>(), ["A", "B"]);
    assert_eq!(dom.at("#a").unwrap().attr("foo"), Some("0"));
    assert!(dom.at("#b").unwrap().attrs().contains_key("myattr"));
    assert_eq!(dom.find("[id]").iter().map(|x| x.attr("id").unwrap()).collect::<Vec<_>>(), ["a", "b"]);
    assert_eq!(dom.to_string(), r#"<div><div foo="0" id="a">A</div><div id="b" myattr>B</div></div>"#);
}

#[test]
fn basic2() {
    // Select based on parent
    let dom = DOM::new(r#"
<body>
<div>test1</div>
<div><div>test2</div></div>
<body>
    "#);
    assert_eq!(dom.find("body > div").get(0).unwrap().text(), "test1");        // right text
    assert_eq!(dom.find("body > div").get(1).unwrap().text(), "");             // no content
    assert_eq!(dom.find("body > div").len(), 2);                               // right number of elements
    assert_eq!(dom.find("body > div > div").get(0).unwrap().text(), "test2");  // right text
    assert_eq!(dom.find("body > div > div").len(), 1);                         // right number of elements
}

#[test]
fn basic3() {
    // Basic navigation
    let dom = DOM::new(r#"
<!doctype foo>
<foo bar="ba&lt;z">
  test
  <simple class="working">easy</simple>
  <test foo="bar" id="test" />
  <!-- lala -->
  works well
  <![CDATA[ yada yada]]>
  <?boom lalalala ?>
  <a little bit broken>
  < very broken
  <br />
  more text
</foo>
    "#);
    assert!(dom.tag().is_none()); // no tag
    assert!(!dom.attrs().contains_key("foo"));
    assert_eq!(
        dom.to_string(),
        r#"
<!DOCTYPE foo>
<foo bar="ba&lt;z">
  test
  <simple class="working">easy</simple>
  <test foo="bar" id="test"></test>
  <!-- lala -->
  works well
  <![CDATA[ yada yada]]>
  <?boom lalalala ?>
  <a bit broken little>
  &lt; very broken
  <br>
  more text
</a></foo>
    "#);

    let simple = dom.at("foo simple.working[class^=\"wor\"]").unwrap();
    assert_eq!(simple.parent().unwrap().text_all(), "test easy works well yada yada < very broken more text");
    assert_eq!(simple.tag().unwrap(), "simple");
    assert_eq!(simple.attr("class").unwrap(), "working");
    assert_eq!(simple.text(), "easy");
    assert_eq!(simple.parent().unwrap().tag().unwrap(), "foo");
    assert_eq!(simple.parent().unwrap().attr("bar").unwrap(), "ba<z");
    assert_eq!(simple.parent().unwrap().childs(None).get(1).unwrap().tag().unwrap(), "test");
    assert_eq!(simple.to_string(), "<simple class=\"working\">easy</simple>");

    assert_eq!(dom.at("test#test").unwrap().tag().unwrap(), "test");
    assert_eq!(dom.at("[class$=\"ing\"]").unwrap().tag().unwrap(), "simple");
    assert_eq!(dom.at("[class$=ing]").unwrap().tag().unwrap(), "simple");
    assert_eq!(dom.at("[class=\"working\"]").unwrap().tag().unwrap(), "simple");
    assert_eq!(dom.at("[class=working][class]").unwrap().tag().unwrap(), "simple");
    assert_eq!(dom.at("foo > simple").unwrap().next().unwrap().tag().unwrap(), "test");
    assert_eq!(dom.at("foo > simple").unwrap().next().unwrap().next().unwrap().tag().unwrap(), "a");
    assert_eq!(dom.at("foo > test").unwrap().prev().unwrap().tag().unwrap(), "simple");
    assert!(dom.next().is_none());
    assert!(dom.prev().is_none());
    assert!(dom.at("foo > a").unwrap().next().is_none());
    assert!(dom.at("foo > simple").unwrap().prev().is_none());
    assert_eq!(dom.at("simple").unwrap().ancestors(None).iter().map(|x| x.tag().unwrap()).collect::<Vec<_>>(), ["foo"]);
}

#[test]
fn class_and_id() {
    // Class and ID
    let dom = DOM::new(r#"<div id="id" class="class">a</div>"#);
    assert_eq!(dom.at("div#id.class").unwrap().text(), "a");
}

#[test]
fn deep_nesting() {
    // Deep nesting (parent combinator)
    let dom = DOM::new(r#"
<html>
  <head>
    <title>Foo</title>
  </head>
  <body>
    <div id="container">
      <div id="header">
        <div id="logo">Hello World</div>
        <div id="buttons">
          <p id="foo">Foo</p>
        </div>
      </div>
      <form>
        <div id="buttons">
          <p id="bar">Bar</p>
        </div>
      </form>
      <div id="content">More stuff</div>
    </div>
  </body>
</html>
    "#);

    let p = dom.find("body > #container > div p[id]");
    assert_eq!(p.len(), 1);
    assert_eq!(p.get(0).unwrap().attr("id").unwrap(), "foo");

    assert_eq!(
        dom.find("div").iter().map(|x| x.attr("id").unwrap()).collect::<Vec<_>>(),
        ["container", "header", "logo", "buttons", "buttons", "content"]
    );
    assert_eq!(
        dom.find("p").iter().map(|x| x.attr("id").unwrap()).collect::<Vec<_>>(),
        ["foo", "bar"]
    );
    assert_eq!(
        dom.at("p").unwrap().ancestors(None).iter().map(|x| x.tag().unwrap()).collect::<Vec<_>>(),
        ["div", "div", "div", "body", "html"]
    );
    assert_eq!(dom.at("html").unwrap().ancestors(None).len(), 0);
    assert_eq!(dom.ancestors(None).len(), 0);
}

#[test]
fn script_tag() {
    let dom = DOM::new(r#"<script charset="utf-8">alert('<hello>world</hello>');</script>"#);
    assert_eq!(dom.at("script").unwrap().text(), "alert('<hello>world</hello>');");
}

#[test]
fn html5_base() {
    // HTML5 (unquoted values)
    let dom = DOM::new(r#"<div id = test foo ="bar" class=tset bar=/baz/ baz=//>works</div>"#);
    assert_eq!(dom.at("#test").unwrap().text(), "works");
    assert_eq!(dom.at("div").unwrap().text(), "works");
    assert_eq!(dom.at("[foo=bar][foo=\"bar\"]").unwrap().text(), "works");
    assert!(dom.at("[foo=\"ba\"]").is_none());
    assert_eq!(dom.at("[foo=bar]").unwrap().text(), "works");
    assert!(dom.at("[foo=ba]").is_none());
    assert_eq!(dom.at(".tset").unwrap().text(), "works");
    assert_eq!(dom.at("[bar=/baz/]").unwrap().text(), "works");
    assert_eq!(dom.at("[baz=//]").unwrap().text(), "works");
}

#[test]
fn html1_mix() {
    // HTML1 (single quotes, uppercase tags and whitespace in attributes)
    let dom = DOM::new(r#"<DIV id = 'test' foo ='bar' class= "tset">works</DIV>"#);
    assert_eq!(dom.at("#test").unwrap().text(), "works");
    assert_eq!(dom.at("div").unwrap().text(), "works");
    assert_eq!(dom.at("[foo=\"bar\"]").unwrap().text(), "works");
    assert!(dom.at("[foo=\"ba\"]").is_none());
    assert_eq!(dom.at("[foo=bar]").unwrap().text(), "works");
    assert!(dom.at("[foo=ba]").is_none());
    assert_eq!(dom.at(".tset").unwrap().text(), "works");
}

#[test]
fn unicode_snowman() {
    // Already decoded Unicode snowman and quotes in selector
    let dom = DOM::new(r#"<div id="snow&apos;m&quot;an">☃</div>"#);
    assert_eq!(dom.at(r#"[id="snow'm\"an"]"#).unwrap().text(), "☃");
    assert_eq!(dom.at(r#"[id="snow'm\22 an"]"#).unwrap().text(), "☃");
    assert_eq!(dom.at(r#"[id="snow\'m\000022an"]"#).unwrap().text(), "☃");
    assert_eq!(dom.at("[id='snow\\'m\"an']").unwrap().text(), "☃");
    assert_eq!(dom.at("[id='snow\\27m\"an']").unwrap().text(), "☃");
    assert!(dom.at(r#"[id="snow'm\22an"]"#).is_none());
    assert!(dom.at(r#"[id="snow'm\21 an"]"#).is_none());
    assert!(dom.at(r#"[id="snow'm\000021an"]"#).is_none());
    assert!(dom.at(r#"[id="snow'm\000021 an"]"#).is_none());
}

#[test]
fn unicode_selectors() {
    // Unicode and escaped selectors
    let html = r#"<html><div id="☃x">Snowman</div><div class="x ♥">Heart</div></html>"#;
    let dom = DOM::new(html);

    assert_eq!(dom.at("#\\\n\\002603x").unwrap().text(),                "Snowman");
    assert_eq!(dom.at("#\\2603 x").unwrap().text(),                     "Snowman");
    assert_eq!(dom.at("#\\\n\\2603 x").unwrap().text(),                 "Snowman");
    assert_eq!(dom.at("[id=\"\\\n\\2603 x\"]").unwrap().text(),         "Snowman");
    assert_eq!(dom.at("[id=\"\\\n\\002603x\"]").unwrap().text(),        "Snowman");
    assert_eq!(dom.at("[id=\"\\\\2603 x\"]").unwrap().text(),           "Snowman");
    assert_eq!(dom.at("html #\\\n\\002603x").unwrap().text(),           "Snowman");
    assert_eq!(dom.at("html #\\2603 x").unwrap().text(),                "Snowman");
    assert_eq!(dom.at("html #\\\n\\2603 x").unwrap().text(),            "Snowman");
    assert_eq!(dom.at("html [id=\"\\\n\\2603 x\"]").unwrap().text(),    "Snowman");
    assert_eq!(dom.at("html [id=\"\\\n\\002603x\"]").unwrap().text(),   "Snowman");
    assert_eq!(dom.at("html [id=\"\\\\2603 x\"]").unwrap().text(),      "Snowman");
    assert_eq!(dom.at("#☃x").unwrap().text(),                           "Snowman");
    assert_eq!(dom.at("html div#☃x").unwrap().text(),                   "Snowman");
    assert_eq!(dom.at("[id^=\"☃\"]").unwrap().text(),                   "Snowman");
    assert_eq!(dom.at("div[id^=\"☃\"]").unwrap().text(),                "Snowman");
    assert_eq!(dom.at("html div[id^=\"☃\"]").unwrap().text(),           "Snowman");
    assert_eq!(dom.at("html > div[id^=\"☃\"]").unwrap().text(),         "Snowman");
    assert_eq!(dom.at("[id^=☃]").unwrap().text(),                       "Snowman");
    assert_eq!(dom.at("div[id^=☃]").unwrap().text(),                    "Snowman");
    assert_eq!(dom.at("html div[id^=☃]").unwrap().text(),               "Snowman");
    assert_eq!(dom.at("html > div[id^=☃]").unwrap().text(),             "Snowman");
    assert_eq!(dom.at(".\\\n\\002665").unwrap().text(),                     "Heart");
    assert_eq!(dom.at(".\\2665").unwrap().text(),                           "Heart");
    assert_eq!(dom.at("html .\\\n\\002665").unwrap().text(),                "Heart");
    assert_eq!(dom.at("html .\\2665").unwrap().text(),                      "Heart");
    assert_eq!(dom.at("html [class$=\"\\\n\\002665\"]").unwrap().text(),    "Heart");
    assert_eq!(dom.at("html [class$=\"\\2665\"]").unwrap().text(),          "Heart");
    assert_eq!(dom.at("[class$=\"\\\n\\002665\"]").unwrap().text(),         "Heart");
    assert_eq!(dom.at("[class$=\"\\2665\"]").unwrap().text(),               "Heart");
    assert_eq!(dom.at(".x").unwrap().text(),                                "Heart");
    assert_eq!(dom.at("html .x").unwrap().text(),                           "Heart");
    assert_eq!(dom.at(".♥").unwrap().text(),                                "Heart");
    assert_eq!(dom.at("html .♥").unwrap().text(),                           "Heart");
    assert_eq!(dom.at("div.♥").unwrap().text(),                             "Heart");
    assert_eq!(dom.at("html div.♥").unwrap().text(),                        "Heart");
    assert_eq!(dom.at("[class$=\"♥\"]").unwrap().text(),                    "Heart");
    assert_eq!(dom.at("div[class$=\"♥\"]").unwrap().text(),                 "Heart");
    assert_eq!(dom.at("html div[class$=\"♥\"]").unwrap().text(),            "Heart");
    assert_eq!(dom.at("html > div[class$=\"♥\"]").unwrap().text(),          "Heart");
    assert_eq!(dom.at("[class$=♥]").unwrap().text(),                        "Heart");
    assert_eq!(dom.at("div[class$=♥]").unwrap().text(),                     "Heart");
    assert_eq!(dom.at("html div[class$=♥]").unwrap().text(),                "Heart");
    assert_eq!(dom.at("html > div[class$=♥]").unwrap().text(),              "Heart");
    assert_eq!(dom.at("[class~=\"♥\"]").unwrap().text(),                    "Heart");
    assert_eq!(dom.at("div[class~=\"♥\"]").unwrap().text(),                 "Heart");
    assert_eq!(dom.at("html div[class~=\"♥\"]").unwrap().text(),            "Heart");
    assert_eq!(dom.at("html > div[class~=\"♥\"]").unwrap().text(),          "Heart");
    assert_eq!(dom.at("[class~=♥]").unwrap().text(),                        "Heart");
    assert_eq!(dom.at("div[class~=♥]").unwrap().text(),                     "Heart");
    assert_eq!(dom.at("html div[class~=♥]").unwrap().text(),                "Heart");
    assert_eq!(dom.at("html > div[class~=♥]").unwrap().text(),              "Heart");
    assert_eq!(dom.at("[class~=\"x\"]").unwrap().text(),                    "Heart");
    assert_eq!(dom.at("div[class~=\"x\"]").unwrap().text(),                 "Heart");
    assert_eq!(dom.at("html div[class~=\"x\"]").unwrap().text(),            "Heart");
    assert_eq!(dom.at("html > div[class~=\"x\"]").unwrap().text(),          "Heart");
    assert_eq!(dom.at("[class~=x]").unwrap().text(),                        "Heart");
    assert_eq!(dom.at("div[class~=x]").unwrap().text(),                     "Heart");
    assert_eq!(dom.at("html div[class~=x]").unwrap().text(),                "Heart");
    assert_eq!(dom.at("html > div[class~=x]").unwrap().text(),              "Heart");
    assert_eq!(dom.at("html").unwrap().to_string(), html);
    assert_eq!(dom.at("#☃x").unwrap().parent().unwrap().to_string(), html);
    assert_eq!(dom.to_string(), html);
    assert_eq!(dom.content(), html);

    let dom = DOM::new(r#"<!DOCTYPE H "-/W/D HT 4/E">☃<title class=test>♥</title>☃"#);
    assert_eq!(dom.at("title").unwrap().text(), "♥");
    assert_eq!(dom.at("*").unwrap().text(), "♥");
    assert_eq!(dom.at(".test").unwrap().text(), "♥");
}

#[test]
fn attrs_on_multiple_lines() {
    // Attributes on multiple lines
    let dom = DOM::new("<div test=23 id='a' \n class='x' foo=bar />");
    assert_eq!(dom.at("div.x").unwrap().attr("test").unwrap(), "23");
    assert_eq!(dom.at("[foo=\"bar\"]").unwrap().attr("class").unwrap(), "x");
}

#[test]
fn markup_chars_in_attr_vals() {
    // Markup characters in attribute values
    let dom = DOM::new("<div id=\"<a>\" \n test='='>Test<div id='><' /></div>");
    assert_eq!(dom.at("div[id=\"<a>\"]").unwrap().attrs().get("test").unwrap().clone(), Some("=".to_owned()));
    assert_eq!(dom.at("[id=\"<a>\"]").unwrap().text(), "Test");
    assert_eq!(dom.at("[id=\"><\"]").unwrap().attrs().get("id").unwrap().clone(), Some("><".to_owned()));
}

#[test]
fn empty_attrs() {
    // Empty attributes
    let dom = DOM::new("<div test=\"\" test2='' />");
    assert_eq!(dom.at("div").unwrap().attr("test").unwrap(), "");
    assert_eq!(dom.at("div").unwrap().attr("test2").unwrap(), "");
    assert_eq!(dom.at("[test]").unwrap().tag().unwrap(), "div");
    assert_eq!(dom.at("[test=\"\"]").unwrap().tag().unwrap(), "div");
    assert_eq!(dom.at("[test2]").unwrap().tag().unwrap(), "div");
    assert_eq!(dom.at("[test2=\"\"]").unwrap().tag().unwrap(), "div");
    assert!(dom.at("[test3]").is_none());
    assert!(dom.at("[test3=\"\"]").is_none());
}

#[test]
fn multi_line_attr() {
    // Multi-line attribute
    let dom = DOM::new("<div class=\"line1\nline2\" />");
    assert_eq!(dom.at("div").unwrap().attr("class").unwrap(), "line1\nline2");
    assert_eq!(dom.at(".line1").unwrap().tag().unwrap(), "div");
    assert_eq!(dom.at(".line2").unwrap().tag().unwrap(), "div");
    assert!(dom.at(".line3").is_none());
}

#[test]
fn entities_in_attrs() {
    assert_eq!(DOM::new("<a href=\"/?foo&lt=bar\"></a>").at("a").unwrap().attr("href").unwrap(), "/?foo&lt=bar");
    assert_eq!(DOM::new("<a href=\"/?f&ltoo=bar\"></a>").at("a").unwrap().attr("href").unwrap(), "/?f&ltoo=bar");
    assert_eq!(DOM::new("<a href=\"/?f&lt-oo=bar\"></a>").at("a").unwrap().attr("href").unwrap(), "/?f<-oo=bar");
    assert_eq!(DOM::new("<a href=\"/?foo=&lt\"></a>").at("a").unwrap().attr("href").unwrap(), "/?foo=<");
    assert_eq!(DOM::new("<a href=\"/?f&lt;oo=bar\"></a>").at("a").unwrap().attr("href").unwrap(), "/?f<oo=bar");
}

#[test]
fn whitespaces_before_closing_bracket() {
    // Whitespaces before closing bracket
    let dom = DOM::new("<div >content</div>");
    assert!(dom.at("div").is_some());
    assert_eq!(dom.at("div").unwrap().text(), "content");
}

#[test]
fn class_with_hyphen() {
    // Class with hyphen
    let dom = DOM::new(r#"<div class="a">A</div><div class="a-1">A1</div>"#);
    assert_eq!(dom.find(".a").iter().map(|x| x.text()).collect::<Vec<_>>(), ["A"]); // found first element only
    assert_eq!(dom.find(".a-1").iter().map(|x| x.text()).collect::<Vec<_>>(), ["A1"]); // found last element only
}

#[test]
fn empty_tags() {
    // Empty tags
    let dom = DOM::new("<hr /><br/><br id=\"br\"/><br />");
    assert_eq!(dom.to_string(), "<hr><br><br id=\"br\"><br>");
}

#[test]
fn inner_html() {
    let dom = DOM::new("<a>xxx<x>x</x>xxx</a>");
    assert_eq!(dom.at("a").unwrap().content(), "xxx<x>x</x>xxx");
    assert_eq!(dom.content(), "<a>xxx<x>x</x>xxx</a>");
}

#[test]
fn multiple_selectors() {
    // Multiple selectors
    let dom = DOM::new("<div id=\"a\">A</div><div id=\"b\">B</div><div id=\"c\">C</div><p>D</p>");
    assert_eq!(dom.find("p, div").iter().map(|x| x.text()).collect::<Vec<_>>(), ["A", "B", "C", "D"]);
    assert_eq!(dom.find("#a, #c").iter().map(|x| x.text()).collect::<Vec<_>>(), ["A", "C"]);
    assert_eq!(dom.find("div#a, div#b").iter().map(|x| x.text()).collect::<Vec<_>>(), ["A", "B"]);
    assert_eq!(dom.find("div[id=\"a\"], div[id=\"c\"]").iter().map(|x| x.text()).collect::<Vec<_>>(), ["A", "C"]);

    let dom2 = DOM::new("<div id=\"☃\">A</div><div id=\"b\">B</div><div id=\"♥x\">C</div>");
    assert_eq!(dom2.find("#☃, #♥x").iter().map(|x| x.text()).collect::<Vec<_>>(), ["A", "C"]);
    assert_eq!(dom2.find("div#☃, div#b").iter().map(|x| x.text()).collect::<Vec<_>>(), ["A", "B"]);
    assert_eq!(dom2.find("div[id=\"☃\"], div[id=\"♥x\"]").iter().map(|x| x.text()).collect::<Vec<_>>(), ["A", "C"]);
}

#[test]
fn multiple_attributes() {
    // Multiple attributes
    let dom = DOM::new(r#"
<div foo="bar" bar="baz">A</div>
<div foo="bar">B</div>
<div foo="bar" bar="baz">C</div>
<div foo="baz" bar="baz">D</div>
    "#);

    assert_eq!(dom.find("div[foo=\"bar\"][bar=\"baz\"]").iter().map(|x| x.text()).collect::<Vec<_>>(), ["A", "C"]);
    assert_eq!(dom.find("div[foo^=\"b\"][foo$=\"r\"]").iter().map(|x| x.text()).collect::<Vec<_>>(), ["A", "B", "C"]);
    assert!(dom.at("[foo=\"bar\"]").unwrap().prev().is_none());
    assert_eq!(dom.at("[foo=\"bar\"]").unwrap().next().unwrap().text(), "B");
    assert_eq!(dom.at("[foo=\"bar\"]").unwrap().next().unwrap().prev().unwrap().text(), "A");
    assert!(dom.at("[foo=\"bar\"]").unwrap().next().unwrap().next().unwrap().next().unwrap().next().is_none());
}

#[test]
fn pseudo_classes() {
    // Pseudo-classes
    let dom = DOM::new(r#"
<form action="/foo">
    <input type="text" name="user" value="test" />
    <input type="checkbox" checked="checked" name="groovy">
    <select name="a">
        <option value="b">b</option>
        <optgroup label="c">
            <option value="d">d</option>
            <option selected="selected" value="e">E</option>
            <option value="f">f</option>
        </optgroup>
        <option value="g">g</option>
        <option selected value="h">H</option>
    </select>
    <input type="submit" value="Ok!" />
    <input type="checkbox" checked name="I">
    <p id="content">test 123</p>
    <p id="no_content"><? test ?><!-- 123 --></p>
</form>
    "#);
    assert_eq!(dom.find(":root").len(), 1);
    assert_eq!(dom.find(":root").get(0).unwrap().tag(), Some("form"));
    assert_eq!(dom.find("*:root").get(0).unwrap().tag(), Some("form"));
    assert_eq!(dom.find("form:root").get(0).unwrap().tag(), Some("form"));
    assert_eq!(dom.find(":checked").len(), 4);
    assert_eq!(dom.find(":checked").get(0).unwrap().attr("name").unwrap(), "groovy");
    assert_eq!(dom.find("option:checked").get(0).unwrap().attr("value").unwrap(), "e");
    assert_eq!(dom.find(":checked").get(1).unwrap().text(), "E");
    assert_eq!(dom.find("*:checked").get(1).unwrap().text(), "E");
    assert_eq!(dom.find(":checked").get(2).unwrap().text(), "H");
    assert_eq!(dom.find(":checked").get(3).unwrap().attr("name").unwrap(), "I");
    assert_eq!(dom.find("option[selected]").len(), 2);
    assert_eq!(dom.find("option[selected]").get(0).unwrap().attr("value").unwrap(), "e");
    assert_eq!(dom.find("option[selected]").get(1).unwrap().text(), "H");
    assert_eq!(dom.find(":checked[value=\"e\"]").get(0).unwrap().text(), "E");
    assert_eq!(dom.find("*:checked[value=\"e\"]").get(0).unwrap().text(), "E");
    assert_eq!(dom.find("option:checked[value=\"e\"]").get(0).unwrap().text(), "E");
    assert_eq!(dom.at("optgroup option:checked[value=\"e\"]").unwrap().text(), "E");
    assert_eq!(dom.at("select option:checked[value=\"e\"]").unwrap().text(), "E");
    assert_eq!(dom.at("select :checked[value=\"e\"]").unwrap().text(), "E");
    assert_eq!(dom.at("optgroup > :checked[value=\"e\"]").unwrap().text(), "E");
    assert_eq!(dom.at("select *:checked[value=\"e\"]").unwrap().text(), "E");
    assert_eq!(dom.at("optgroup > *:checked[value=\"e\"]").unwrap().text(), "E");
    assert_eq!(dom.find(":checked[value=\"e\"]").len(), 1);
    assert_eq!(dom.find(":empty").get(0).unwrap().attr("name").unwrap(), "user");
    assert_eq!(dom.find("input:empty").get(0).unwrap().attr("name").unwrap(), "user");
    assert_eq!(dom.at(":empty[type^=\"ch\"]").unwrap().attr("name").unwrap(), "groovy");
    assert_eq!(dom.at("p").unwrap().attr("id").unwrap(), "content");
    assert_eq!(dom.at("p:empty").unwrap().attr("id").unwrap(), "no_content");

    // More pseudo-classes
    let dom = DOM::new("
<ul>
    <li>A</li>
    <li>B</li>
    <li>C</li>
    <li>D</li>
    <li>E</li>
    <li>F</li>
    <li>G</li>
    <li>H</li>
</ul>
    ");
    assert_eq!(dom.find("li:nth-child(odd)").iter().map(|x| x.text()).collect::<Vec<_>>(), ["A", "C", "E", "G"]);
    assert_eq!(dom.find("li:NTH-CHILD(ODD)").iter().map(|x| x.text()).collect::<Vec<_>>(), ["A", "C", "E", "G"]);
    assert_eq!(dom.find("li:nth-last-child(odd)").iter().map(|x| x.text()).collect::<Vec<_>>(), ["B", "D", "F", "H"]);
    assert_eq!(dom.find(":nth-child(odd)").get(0).unwrap().tag().unwrap(), "ul");
    assert_eq!(dom.find(":nth-child(odd)").get(1).unwrap().text(), "A");
    assert_eq!(dom.find(":nth-child(1)").get(0).unwrap().tag().unwrap(), "ul");
    assert_eq!(dom.find(":nth-child(1)").get(1).unwrap().text(), "A");
    assert_eq!(dom.find(":nth-last-child(odd)").get(0).unwrap().tag().unwrap(), "ul");
    assert_eq!(dom.find(":nth-last-child(odd)").last().unwrap().text(), "H");
    assert_eq!(dom.find(":nth-last-child(1)").get(0).unwrap().tag().unwrap(), "ul");
    assert_eq!(dom.find(":nth-last-child(1)").get(1).unwrap().text(), "H");
    assert_eq!(dom.find("li:nth-child(2n+1)").iter().map(|x| x.text()).collect::<Vec<_>>(), ["A", "C", "E", "G"]);
    assert_eq!(dom.find("li:nth-child(2n + 1)").iter().map(|x| x.text()).collect::<Vec<_>>(), ["A", "C", "E", "G"]);
    assert_eq!(dom.find("li:nth-last-child(2n+1)").iter().map(|x| x.text()).collect::<Vec<_>>(), ["B", "D", "F", "H"]);
    assert_eq!(dom.find("li:nth-child(even)").iter().map(|x| x.text()).collect::<Vec<_>>(), ["B", "D", "F", "H"]);
    assert_eq!(dom.find("li:NTH-CHILD(EVEN)").iter().map(|x| x.text()).collect::<Vec<_>>(), ["B", "D", "F", "H"]);
    assert_eq!(dom.find("li:nth-last-child( even )").iter().map(|x| x.text()).collect::<Vec<_>>(), ["A", "C", "E", "G"]);
    assert_eq!(dom.find("li:nth-child(2n+2)").iter().map(|x| x.text()).collect::<Vec<_>>(), ["B", "D", "F", "H"]);
    assert_eq!(dom.find("li:nTh-chILd(2N+2)").iter().map(|x| x.text()).collect::<Vec<_>>(), ["B", "D", "F", "H"]);
    assert_eq!(dom.find("li:nth-child( 2n + 2 )").iter().map(|x| x.text()).collect::<Vec<_>>(), ["B", "D", "F", "H"]);
    assert_eq!(dom.find("li:nth-last-child(2n+2)").iter().map(|x| x.text()).collect::<Vec<_>>(), ["A", "C", "E", "G"]);
    assert_eq!(dom.find("li:nth-child(4n+1)").iter().map(|x| x.text()).collect::<Vec<_>>(), ["A", "E"]);
    assert_eq!(dom.find("li:nth-last-child(4n+1)").iter().map(|x| x.text()).collect::<Vec<_>>(), ["D", "H"]);
    assert_eq!(dom.find("li:nth-child(4n+4)").iter().map(|x| x.text()).collect::<Vec<_>>(), ["D", "H"]);
    assert_eq!(dom.find("li:nth-last-child(4n+4)").iter().map(|x| x.text()).collect::<Vec<_>>(), ["A", "E"]);
    assert_eq!(dom.find("li:nth-child(4n)").iter().map(|x| x.text()).collect::<Vec<_>>(), ["D", "H"]);
    assert_eq!(dom.find("li:nth-child( 4n )").iter().map(|x| x.text()).collect::<Vec<_>>(), ["D", "H"]);
    assert_eq!(dom.find("li:nth-last-child(4n)").iter().map(|x| x.text()).collect::<Vec<_>>(), ["A", "E"]);
    assert_eq!(dom.find("li:nth-child(5n-2)").iter().map(|x| x.text()).collect::<Vec<_>>(), ["C", "H"]);
    assert_eq!(dom.find("li:nth-child( 5n - 2 )").iter().map(|x| x.text()).collect::<Vec<_>>(), ["C", "H"]);
    assert_eq!(dom.find("li:nth-last-child(5n-2)").iter().map(|x| x.text()).collect::<Vec<_>>(), ["A", "F"]);
    assert_eq!(dom.find("li:nth-child(-n+3)").iter().map(|x| x.text()).collect::<Vec<_>>(), ["A", "B", "C"]);
    assert_eq!(dom.find("li:nth-child( -n + 3 )").iter().map(|x| x.text()).collect::<Vec<_>>(), ["A", "B", "C"]);
    assert_eq!(dom.find("li:nth-last-child(-n+3)").iter().map(|x| x.text()).collect::<Vec<_>>(), ["F", "G", "H"]);
    assert_eq!(dom.find("li:nth-child(-1n+3)").iter().map(|x| x.text()).collect::<Vec<_>>(), ["A", "B", "C"]);
    assert_eq!(dom.find("li:nth-last-child(-1n+3)").iter().map(|x| x.text()).collect::<Vec<_>>(), ["F", "G", "H"]);
    assert_eq!(dom.find("li:nth-child(3n)").iter().map(|x| x.text()).collect::<Vec<_>>(), ["C", "F"]);
    assert_eq!(dom.find("li:nth-last-child(3n)").iter().map(|x| x.text()).collect::<Vec<_>>(), ["C", "F"]);
    assert_eq!(dom.find("li:NTH-LAST-CHILD(3N)").iter().map(|x| x.text()).collect::<Vec<_>>(), ["C", "F"]);
    assert_eq!(dom.find("li:Nth-Last-Child(3N)").iter().map(|x| x.text()).collect::<Vec<_>>(), ["C", "F"]);
    assert_eq!(dom.find("li:nth-child( 3 )").iter().map(|x| x.text()).collect::<Vec<_>>(), ["C"]);
    assert_eq!(dom.find("li:nth-last-child( +3 )").iter().map(|x| x.text()).collect::<Vec<_>>(), ["F"]);
    assert_eq!(dom.find("li:nth-child(1n+0)").iter().map(|x| x.text()).collect::<Vec<_>>(), ["A", "B", "C", "D", "E", "F", "G"]);
    assert_eq!(dom.find("li:nth-child(1n-0)").iter().map(|x| x.text()).collect::<Vec<_>>(), ["A", "B", "C", "D", "E", "F", "G"]);
    assert_eq!(dom.find("li:nth-child(n+0)").iter().map(|x| x.text()).collect::<Vec<_>>(), ["A", "B", "C", "D", "E", "F", "G"]);
    assert_eq!(dom.find("li:nth-child(n)").iter().map(|x| x.text()).collect::<Vec<_>>(), ["A", "B", "C", "D", "E", "F", "G"]);
    assert_eq!(dom.find("li:nth-child(n+0)").iter().map(|x| x.text()).collect::<Vec<_>>(), ["A", "B", "C", "D", "E", "F", "G"]);
    assert_eq!(dom.find("li:NTH-CHILD(N+0)").iter().map(|x| x.text()).collect::<Vec<_>>(), ["A", "B", "C", "D", "E", "F", "G"]);
    assert_eq!(dom.find("li:Nth-Child(N+0)").iter().map(|x| x.text()).collect::<Vec<_>>(), ["A", "B", "C", "D", "E", "F", "G"]);
    assert_eq!(dom.find("li:nth-child(n)").iter().map(|x| x.text()).collect::<Vec<_>>(), ["A", "B", "C", "D", "E", "F", "G"]);
    assert_eq!(dom.find("li:nth-child(0n+1)").iter().map(|x| x.text()).collect::<Vec<_>>(), ["A"]);
    assert_eq!(dom.find("li:nth-child(0n+0)").len(), 0);
    assert_eq!(dom.find("li:nth-child(0)").len(), 0);
    assert_eq!(dom.find("li:nth-child()").len(), 0);
    assert_eq!(dom.find("li:nth-child(whatever)").len(), 0);
    assert_eq!(dom.find("li:whatever(whatever)").len(), 0);

    // Even more pseudo-classes
    let dom = DOM::new(r#"
<ul>
    <li>A</li>
    <p>B</p>
    <li class="test ♥">C</li>
    <p>D</p>
    <li>E</li>
    <li>F</li>
    <p>G</p>
    <li>H</li>
    <li>I</li>
</ul>
<div>
    <div class="☃">J</div>
</div>
<div>
    <a href="http://mojolicious.org">Mojo!</a>
    <div class="☃">K</div>
    <a href="http://mojolicious.org">Mojolicious!</a>
</div>
    "#);
    assert_eq!(dom.find("ul :nth-child(odd)").iter().map(|x| x.text()).collect::<Vec<_>>(), ["A", "C", "E", "G", "I"]);
    assert_eq!(dom.find("li:nth-of-type(odd)").iter().map(|x| x.text()).collect::<Vec<_>>(), ["A", "E", "H"]);
    assert_eq!(dom.find("li:nth-last-of-type( odd )").iter().map(|x| x.text()).collect::<Vec<_>>(), ["C", "F", "I"]);
    assert_eq!(dom.find("p:nth-of-type(odd)").iter().map(|x| x.text()).collect::<Vec<_>>(), ["B", "G"]);
    assert_eq!(dom.find("p:nth-last-of-type(odd)").iter().map(|x| x.text()).collect::<Vec<_>>(), ["B", "G"]);
    assert_eq!(dom.find("ul :nth-child(1)").iter().map(|x| x.text()).collect::<Vec<_>>(), ["A"]);
    assert_eq!(dom.find("ul :first-child").iter().map(|x| x.text()).collect::<Vec<_>>(), ["A"]);
    assert_eq!(dom.find("p:nth-of-type(1)").iter().map(|x| x.text()).collect::<Vec<_>>(), ["B"]);
    assert_eq!(dom.find("p:first-of-type").iter().map(|x| x.text()).collect::<Vec<_>>(), ["B"]);
    assert_eq!(dom.find("li:nth-of-type(1)").iter().map(|x| x.text()).collect::<Vec<_>>(), ["A"]);
    assert_eq!(dom.find("li:first-of-type").iter().map(|x| x.text()).collect::<Vec<_>>(), ["A"]);
    assert_eq!(dom.find("ul :nth-last-child(-n+1)").iter().map(|x| x.text()).collect::<Vec<_>>(), ["I"]);
    assert_eq!(dom.find("ul :last-child").iter().map(|x| x.text()).collect::<Vec<_>>(), ["I"]);
    assert_eq!(dom.find("p:nth-last-of-type(-n+1)").iter().map(|x| x.text()).collect::<Vec<_>>(), ["G"]);
    assert_eq!(dom.find("p:last-of-type").iter().map(|x| x.text()).collect::<Vec<_>>(), ["G"]);
    assert_eq!(dom.find("li:nth-last-of-type(-n+1)").iter().map(|x| x.text()).collect::<Vec<_>>(), ["I"]);
    assert_eq!(dom.find("li:last-of-type").iter().map(|x| x.text()).collect::<Vec<_>>(), ["I"]);
    assert_eq!(dom.find("ul :nth-child(-n+3):not(li)").iter().map(|x| x.text()).collect::<Vec<_>>(), ["B"]);
    assert_eq!(dom.find("ul :nth-child(-n+3):NOT(li)").iter().map(|x| x.text()).collect::<Vec<_>>(), ["B"]);
    assert_eq!(dom.find("ul :nth-child(-n+3):not(:first-child)").iter().map(|x| x.text()).collect::<Vec<_>>(), ["B", "C"]);
    assert_eq!(dom.find("ul :nth-child(-n+3):not(.♥)").iter().map(|x| x.text()).collect::<Vec<_>>(), ["A", "B"]);
    assert_eq!(dom.find("ul :nth-child(-n+3):not([class$=\"♥\"])").iter().map(|x| x.text()).collect::<Vec<_>>(), ["A", "B"]);
    assert_eq!(dom.find("ul :nth-child(-n+3):not(li[class$=\"♥\"])").iter().map(|x| x.text()).collect::<Vec<_>>(), ["A", "B"]);
    assert_eq!(dom.find("ul :nth-child(-n+3):not([class$=\"♥\"][class^=\"test\"])").iter().map(|x| x.text()).collect::<Vec<_>>(), ["A", "B"]);
    assert_eq!(dom.find("ul :nth-child(-n+3):not(*[class$=\"♥\"])").iter().map(|x| x.text()).collect::<Vec<_>>(), ["A", "B"]);
    assert_eq!(dom.find("ul :nth-child(-n+3):not(:nth-child(-n+2))").iter().map(|x| x.text()).collect::<Vec<_>>(), ["C"]);
    assert_eq!(dom.find("ul :nth-child(-n+3):not(:nth-child(1)):not(:nth-child(2))").iter().map(|x| x.text()).collect::<Vec<_>>(), ["C"]);
    assert_eq!(dom.find(":only-child").iter().map(|x| x.text()).collect::<Vec<_>>(), ["J"]);
    assert_eq!(dom.find("div :only-of-type").iter().map(|x| x.text()).collect::<Vec<_>>(), ["J", "K"]);
    assert_eq!(dom.find("div:only-child").iter().map(|x| x.text()).collect::<Vec<_>>(), ["J"]);
    assert_eq!(dom.find("div div:only-of-type").iter().map(|x| x.text()).collect::<Vec<_>>(), ["J", "K"]);
}
