# Colors

**All colors from the color crate are in the sRGB color space.**

### Wgpu:
When the surface is created we detect if the texture format is sRGB and store the boolean in our global uniform.

#### Path Renderer:
Vertex Input: sRGB (because of the color crate).

If the surface is sRGB we convert our input to a linear color, the surface will convert this back to sRGB later.
Otherwise, leave the color alone because it is already in a sRGB format and the surface will not do any more transformations.
#### Text Renderer:
For the text color (vertex color):
 - Vertex Input: sRGB (because of the color crate).
 - If the surface is sRGB we convert our input to a linear color, the surface will convert this back to sRGB later.
 - Otherwise, leave the color alone because it is already in a sRGB format and the surface will not do any more transformations.

For the sampled emoji texel:
- Always convert the sampled emoji color to sRGB and on output we'll use the same process that we use for text colors.

#### Image Renderer:
Sampled Texel: Always linear.

If the surface is sRGB do nothing, the surface will convert our linear color to sRGB later.
Otherwise, convert our linear color to sRGB because the surface is linear and will not do any more transformations.