/// LuaTalk output Typst code.
/// Works with a JSON output file from LuaTalk.
/// Use `typst` to render this file.
///
/// Visit the crate homepage `https://github.com/celet-ff-io/luatalk-cli`
/// for more information about LuaTalk.
///
/// Copyright (c) 2026-present celet-ff-io
/// Licensed under the MIT License or the Apache License 2.0
/// Version 0.1.0

// ====================
// Function definitions
// ====================

#let interval(
  is-first: false,
  length-factor: 100%,
) = {
  let length-unit = length-factor * 1em
  if is-first {
    v(0.5 * length-unit)
  } else {
    v(0.25 * length-unit)
  }
}

// Should be a sans font at least
#let _default-font = "Noto Sans"

#let _text-bubble(
  is-first: false,
  side: "left",
  bg-color: rgb("#000000"),
  font: _default-font,
  length-factor: 100%,
  content,
) = {
  // Note that the bubble-height is estimated as 1.70 * length-unit
  assert(
    side in ("left", "right"),
  )
  let length-unit = length-factor * 1em
  let text-rect = rect(
    fill: bg-color,
    inset: 0.5 * length-unit,
    radius: 0.313 * length-unit, // bubble-height * 16 / 87
    stroke: bg-color,
    text(
      fill: white,
      size: length-unit,
      font: font,
      content,
    ),
  )
  let add-effect = if is-first {
    if side == "left" {
      polygon(
        fill: bg-color,
        (0.254 * length-unit, 0.469 * length-unit), // (bubble-height * 13 / 87, bubble-height * 24 / 87)
        (0em, 0.625 * length-unit), // (0, bubble-height * 32 / 87)
        (0.254 * length-unit, 0.782 * length-unit), // (bubble-height * 13 / 87, bubble-height * 40 / 87))
      )
    } else if side == "right" {
      polygon(
        fill: bg-color,
        (0em, 0.469 * length-unit), // (0, bubble-height * 24 / 87)
        (0.254 * length-unit, 0.625 * length-unit), // (bubble-height * 13 / 87, bubble-height * 32 / 87)
        (0em, 0.782 * length-unit), // (0, bubble-height * 40 / 87))
      )
    }
  } else {
    h(0.254 * length-unit)
  }
  if side == "left" {
    stack(
      dir: ltr,
      add-effect,
      text-rect,
    )
  } else if side == "right" {
    stack(
      dir: ltr,
      text-rect,
      add-effect,
    )
  }
}

#let _image-bubble(
  length-factor: 100%,
  content,
) = {
  set block(above: 0em, below: 0em)
  let length-unit = length-factor * 1em
  let border-color = rgb("#dce0e3")
  pad(
    left: 0.254 * length-unit,
    top: 0.254 * length-unit,
    rect(
      inset: 0.5 * length-unit,
      radius: 0.313 * length-unit,
      stroke: 0.059 * length-unit // bubble-height * 3 / 87
        + border-color,
      width: 11.275 * length-unit, // bubble-height * 577 / 87
      content,
    ),
  )
}

#let guest-section(
  is-first: false,
  name: none,
  avatar-path: none,
  type: "text",
  font: _default-font,
  length-factor: 100%,
  content,
) = {
  // Note that the estimated bubble-height is 1.70 * length-unit
  assert(
    type in ("text", "image"),
  )
  set block(above: 0em, below: 0em)
  let length-unit = length-factor * 1em
  let guest-name-color = rgb("#3f444a")
  let guest-color = rgb("4c5b6f")
  let avatar-size = 2.96 * length-unit
  stack(
    dir: ltr,
    spacing: 0.371 * length-unit, // bubble-height * 19 / 87
    if is-first and avatar-path != none {
      box(
        width: avatar-size,
        height: avatar-size,
        radius: 50%,
        clip: true,
        image(avatar-path, width: avatar-size, height: avatar-size, fit: "cover"),
      )
    } else {
      box(width: avatar-size)
    },
    stack(
      dir: ttb,
      spacing: 0.4 * length-unit,

      if is-first {
        pad(
          left: 0.254 * length-unit,
          top: 0.15 * length-unit,
          text(
            fill: guest-name-color,
            size: 0.941 * length-unit, // length-unit * 32 / 34
            font: font,
            weight: "bold",
            name,
          ),
        )
      } else { v(0em) },

      if type == "text" {
        _text-bubble(
          is-first: is-first,
          side: "left",
          bg-color: guest-color,
          font: font,
          length-factor: length-factor,
          content,
        )
      } else {
        _image-bubble(
          length-factor: length-factor,
          content,
        )
      },
    ),
  )
}

#let host-bubble(
  is-first: false,
  type: "text",
  font: _default-font,
  length-factor: 100%,
  content,
) = {
  // Note that the estimated bubble-height is 1.70 * length-unit
  assert(
    type in ("text", "image"),
  )
  set block(above: 0em, below: 0em)
  let length-unit = length-factor * 1em
  let host-color = rgb("#4a8acb")
  align(
    top + right,
    if type == "text" {
      _text-bubble(
        is-first: is-first,
        side: "right",
        bg-color: host-color,
        font: font,
        length-factor: length-factor,
        content,
      )
    } else if type == "image" {
      _image-bubble(
        length-factor: length-factor,
        content,
      )
    },
  )
}

#let system-bubble(font: _default-font, length-factor: 100%, content) = {
  set block(above: 0em, below: 0em)
  let length-unit = length-factor * 1em
  rect(
    width: 100%,
    fill: rgb("#e1e7ec"),
    radius: 0.313 * length-unit,
    inset: 0.5 * length-unit,
    text(
      fill: rgb("#2a323e"),
      size: length-unit,
      font: font,
      align(center, content),
    ),
  )
}

#let _choices-bubble(
  title: "Title",
  bg-color: white,
  border-color: black,
  bar-color: black,
  button-bg-color: white,
  button-text-color: white,
  font: _default-font,
  length-factor: 100%,
  content,
) = {
  set block(above: 0em, below: 0em)
  let length-unit = length-factor * 1em
  let choice-buttons = content
    .split("\n")
    .map(line => line.trim())
    .filter(line => line != "")
    .map(line => align(center, block(
      rect(
        width: 100%,
        fill: button-bg-color,
        inset: 0.5 * length-unit,
        radius: 0.39 * length-unit,
        stroke: 0.039 * length-unit + rgb("#b4bcc0"),
        text(
          fill: button-text-color,
          size: length-unit,
          font: font,
          weight: "bold",
          align(center, line),
        ),
      ),
    )))
  pad(
    left: 3.214 * length-unit,
    block(
      clip: true,
      fill: bg-color,
      radius: 0.332 * length-unit,
      stroke: 0.059 * length-unit + border-color,
      inset: (
        top: 0.508 * length-unit,
        left: 0.645 * length-unit,
        bottom: 0.508 * length-unit,
        right: 0.645 * length-unit,
      ),
      {
        stack(
          dir: ttb,
          spacing: 0.430 * length-unit,

          stack(
            dir: ttb,
            spacing: 0.371 * length-unit,

            stack(
              dir: ltr,
              spacing: 0.312 * length-unit,
              rect(
                fill: bar-color,
                height: 0.879 * length-unit,
                width: 0.117 * length-unit,
              ),
              pad(
                top: 0.09 * length-unit,
                text(
                  fill: rgb("#4c5b6f"),
                  size: length-unit,
                  font: font,
                  weight: "bold",
                  title,
                ),
              ),
            ),
            rect(
              fill: rgb("#acbbc1"),
              height: 0.039 * length-unit,
              width: 100%,
            ),
          ),

          ..choice-buttons,
        )
      },
    ),
  )
}

#let reply-bubble(
  lang: "en",
  font: _default-font,
  length-factor: 100%,
  content,
) = {
  let title = if lang == "en" { "Reply" } else if lang == "ja" { "返信する" } else if lang == "ko" {
    "답장"
  } else if lang == "zh-Hans" { "回复" } else if lang == "zh-Hant" { "回覆" }
  _choices-bubble(
    title: title,
    bg-color: rgb("#e2edf1"),
    border-color: rgb("#ced8dc"),
    bar-color: rgb("#2799e5"),
    button-bg-color: white,
    button-text-color: rgb("#5e6a7a"),
    font: font,
    length-factor: length-factor,
    content,
  )
}

#let bond-story-bubble(
  lang: "en",
  font: _default-font,
  length-factor: 100%,
  content,
) = {
  let title = if lang == "en" { "Relationship Event" } else if lang == "ja" { "絆イベント" } else if lang == "ko" {
    "이야기 이벤트"
  } else if lang == "zh-Hans" { "羁绊剧情" } else if lang == "zh-Hant" { "羈絆劇情" }
  _choices-bubble(
    title: title,
    bg-color: rgb("#ffedf1"),
    border-color: rgb("#ddd3d9"),
    bar-color: rgb("#fc8da2"),
    button-bg-color: rgb("#fe718c"),
    button-text-color: white,
    font: font,
    length-factor: length-factor,
    content,
  )
}

/// May be the only function you need to call
#let article-render(width: 720pt, font: _default-font, length-factor: 100%, article) = {
  set page(width: width, height: auto)
  let lang = article.lang
  let pages = article.pages.map(page => {
    let body-content(body) = {
      let (type, value) = body
      if type == "text" {
        value.content
      } else if type == "image" {
        image(value.path)
      }
    }
    let text-body-content(body) = if body.type == "text" {
      body.value.content
    }

    let (last_role, last_type) = (none, none)
    for msg in page.msgs {
      let (role, body, profile) = msg
      let type = body.type
      // Once the role changes or type changes, consider it a "first message"
      let get-is-first(role) = last_role != role or last_type != type
      if role == "guest" {
        let is-first = get-is-first(role)
        let (name, avatar-path) = if is-first {
          (profile.name, profile.avatar.path)
        } else {
          (none, none)
        }
        interval(is-first: is-first, length-factor: length-factor)
        guest-section(
          name: name,
          avatar-path: avatar-path,
          type: type,
          is-first: is-first,
          font: font,
          length-factor: length-factor,
          body-content(body),
        )
      } else if role == "host" {
        let is-first = get-is-first(role)
        interval(is-first: is-first, length-factor: length-factor)
        host-bubble(
          type: type,
          is-first: get-is-first(role),
          font: font,
          length-factor: length-factor,
          body-content(body),
        )
      } else if role == "system" {
        interval(length-factor: length-factor)
        system-bubble(font: font, length-factor: length-factor, text-body-content(body))
      } else if role == "reply" {
        interval(length-factor: length-factor)
        reply-bubble(lang: lang, font: font, length-factor: length-factor, text-body-content(body))
      } else if role == "bond_story" {
        interval(length-factor: length-factor)
        bond-story-bubble(lang: lang, font: font, length-factor: length-factor, text-body-content(body))
      }
      (last_role, last_type) = (role, type)
    }
  })
  pages.join(pagebreak())
}

// =========================
// Edit your code below
// =========================

// Example
//
// // Your article data in JSON format, output from LuaTalk.
// #let article = json("output.json")
// // Text base size;
// // Our code above is using `em` and the actual size of text is base on your setting here.
// #set text(20pt)
// // Page width.
// #let width = 720pt
// // Set font family you want to use.
// // You may download font you want to use and make `typst` able to find it.
// // #let font = "Noto Sans" // An example fallback font if you do not get the following one
// #let font = "BlueakaBetaGBK"
// // Length factor, for zooming allow elements
// #let length-factor = 100%
//
// #article-render(
//   article,
//   width: width,
//   font: font,
//   length-factor: length-factor,
// )
