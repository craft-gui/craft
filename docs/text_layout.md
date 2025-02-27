# Text Layout

Spans:
- Contain strings of text or other inline elements (images).
- Only valid inside of Text or Span elements.

Text:
- Contains text.
- Spans can be added to the text.
- Is responsible for computing the inline layout of the enclosed text, including their spans.

---
 
## Text Element Compute Layout
1. Constructs a `Parley` text `Layout`.
2. Builds the layout by collecting the text of spans and their inline boxes. 
3. Computes the `Layout`.
4. Fills the children spans with their x, y, width, and height.

## Span Element Compute Layout
No operation should be performed.

---

## Text Element Draw
1. Draw all text from the built `Parley` layout. 
2. Recursively call draw on children span elements.

## Span Element Draw
1. Recursively call draw on children inline elements.