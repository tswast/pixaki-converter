# Tools

This document describes the tools available in Pixel Studio Pro and how their actions are stored and applied to the image data.

## Tool List

The following tools are defined in the `Tool` enum (indices in parentheses):

| ID | Name | Description |
| :--- | :--- | :--- |
| 0 | `Pen` | Basic drawing tool. |
| 1 | `Pipette` | Color picker. Usually UI-only. |
| 2 | `Eraser` | Removes pixels. |
| 3 | `Fill` | Flood fill (Paint Bucket). |
| 4 | `MoveCamera` | UI tool for panning. |
| 5 | `GenericTool` | Placeholder for multi-mode tools. |
| 6 | `Clear` | Clears a selection or the entire layer. |
| 7 | `Copy` | UI-only. |
| 8 | `Cut` | Clears selection and copies to clipboard. |
| 9 | `Paste` | (Obsolete) Pasting logic. |
| 10 | `Move` | Moves pixels by an offset. |
| 11 | `MirrorByX` | Mirrors selection horizontally. |
| 12 | `MirrorByY` | Mirrors selection vertically. |
| 13 | `FlipByX` | Flips selection horizontally. |
| 14 | `FlipByY` | Flips selection vertically. |
| 15 | `RotateLeft` | Rotates selection 90° counter-clockwise. |
| 16 | `RotateRight` | Rotates selection 90° clockwise. |
| 17 | `DotPen` | Single pixel pen. |
| 18 | `ReplaceColor` | Replaces a specific color globally or in selection. |
| 19 | `EraserPen` | Toggles between drawing and erasing. |
| 20 | `PasteImage` | Pastes image data with optional mask. |
| 21 | `RotateRect` | Rotates a rectangular selection by an arbitrary angle. |
| 22 | `DitheringPen` | Pen with dithering patterns. |
| 23 | `MagicWand` | Selection tool. |
| 24 | `ColorAdjustment` | HSL adjustment. |
| 25 | `Brush` | Advanced brush tool. |
| 26 | `PixelSelect` | Selection tool. |
| 27 | `Lasso` | Selection tool. |
| 28 | `Cursor` | UI tool. |
| 29 | `OutlineTool` | Generates outlines. |

## Tool Application Logic (Pseudocode)

The following pseudocode describes how `History.PerformAction` applies each tool to the pixel buffer.

### Pen / DotPen / DitheringPen / Brush
```python
def apply_drawing_tool(pixels, action):
    # Used by Pen, DotPen, DitheringPen, and Brush.
    # These tools record specific pixel changes in action.Positions.
    
    if len(action.Colors) == 1:
        # Optimized path: all pixels have the same color
        for pos in action.Positions:
            pixels.set_pixel(pos.X, pos.Y, action.Colors[0])
    else:
        # Each position has a corresponding color index
        for i, pos in enumerate(action.Positions):
            color = action.Colors[action.ColorIndexes[i]]
            pixels.set_pixel(pos.X, pos.Y, color)
```

### Eraser
```python
def apply_eraser(pixels, action):
    # Similar to Pen, but sets pixels to transparent black (0, 0, 0, 0)
    for pos in action.Positions:
        pixels.set_pixel(pos.X, pos.Y, transparent_black)
```

### EraserPen
```python
def apply_eraser_pen(pixels, action):
    # Toggles between drawing and erasing based on current pixel state
    for pos in action.Positions:
        current_color = pixels.get_pixel(pos.X, pos.Y)
        if current_color.a == 0:
            # If empty, draw with the first color in the palette
            pixels.set_pixel(pos.X, pos.Y, action.Colors[0])
        else:
            # If not empty, erase
            pixels.set_pixel(pos.X, pos.Y, transparent_black)
```

### OutlineTool
```python
def apply_outline(pixels, action):
    # The OutlineTool typically generates a set of pixel changes
    # that are stored and applied identically to the Pen tool.
    apply_drawing_tool(pixels, action)
```

### Selection Tools (MagicWand / PixelSelect / Lasso)
These tools typically do not modify the pixel buffer directly via `History`. Instead, they modify the document's selection state. If they are stored in history, they likely store the resulting selection mask in `action.Positions`.

### Fill
```python
def apply_fill(pixels, action):
    # action.Positions[0] is the start point
    # action.Colors[0] is the fill color
    # action.Float is the tolerance
    TextureHelper.fill_texture(pixels, action.Positions[0], action.Colors[0], action.Float)
```

### ReplaceColor
```python
def apply_replace_color(pixels, action):
    # action.Positions[0] is the reference pixel to pick the color to replace
    # action.Colors[0] is the new color
    # action.Float is the tolerance
    target_color = pixels.get_pixel(action.Positions[0])
    TextureHelper.replace_color(pixels, target_color, action.Colors[0], action.Float)
```

### Clear / Cut
```python
def apply_clear(pixels, action):
    if len(action.Positions) > 0:
        # Clear specific pixels
        for pos in action.Positions:
            pixels.set_pixel(pos.X, pos.Y, transparent_black)
    else:
        # Clear selection rectangle
        rect = action.Rect # parsed from Meta
        for y in range(rect.Y, rect.Y + rect.Height):
            for x in range(rect.X, rect.X + rect.Width):
                pixels.set_pixel(x, y, transparent_black)
```

### Paste / PasteImage
```python
def apply_paste_image(pixels, action):
    paste_data = action.PasteAction # parsed from Meta
    
    # If it has a mask, clear the masked area first
    if paste_data.has_mask:
        for pos in paste_data.mask:
            pixels.set_pixel(pos.X, pos.Y, transparent_black)
    elif paste_data.has_rect_source:
        # Clear source rect area
        rect = paste_data.rect_source
        clear_rect(pixels, rect)
        
    # Paste the buffer
    rect = paste_data.rect
    for y in range(rect.Height):
        for x in range(rect.Width):
            dest_x = x + rect.X
            dest_y = y + rect.Y
            if is_in_bounds(dest_x, dest_y):
                color = paste_data.buffer[x + y * rect.Width]
                if color.a > 0:
                    pixels.set_pixel(dest_x, dest_y, color)
```

### Move
```python
def apply_move(pixels, action):
    offset = action.Positions[1] - action.Positions[0]
    
    # Move uses a buffer of the state before the first move in a sequence
    source_buffer = history.get_move_buffer() 
    
    if len(action.Positions) == 2:
        # Move a rectangular selection
        rect = action.Rect
        # Clear original rect in current pixels
        clear_rect(pixels, rect)
        # Draw from buffer to new position
        for y in range(canvas_height):
            for x in range(canvas_width):
                dest_x = x + offset.X
                dest_y = y + offset.Y
                color = source_buffer.get_pixel(x, y)
                if rect.contains(x, y) and color.a > 0:
                    pixels.set_pixel_safe(dest_x, dest_y, color)
    else:
        # Move specific pixels (Positions[2:])
        for i in range(2, len(action.Positions)):
            pos = action.Positions[i]
            pixels.set_pixel(pos.X, pos.Y, transparent_black)
        for i in range(2, len(action.Positions)):
            pos = action.Positions[i]
            dest_x = pos.X + offset.X
            dest_y = pos.Y + offset.Y
            pixels.set_pixel_safe(dest_x, dest_y, source_buffer.get_pixel(pos.X, pos.Y))
```

### Mirror / Flip
```python
# Mirror creates a copy across the axis
# Flip swaps pixels across the axis

def apply_mirror(pixels, action):
    rect = action.Rect
    for y in range(rect.Height):
        for x in range(rect.Width):
            color = pixels.get_pixel(x + rect.X, y + rect.Y)
            if color.a > 0:
                if action.Tool == MirrorByX:
                    dest_x = rect.X + (2 * rect.Width - x - 1)
                    dest_y = rect.Y + y
                else: # MirrorByY
                    dest_x = rect.X + x
                    dest_y = rect.Y + (-y - 1) # Note: Logic in code seems specific to rect coordinate system
                pixels.set_pixel_safe(dest_x, dest_y, color)

def apply_flip_x(pixels, action):
    rect = action.Rect
    for y in range(rect.Height):
        for x in range(rect.Width // 2):
            left_pos = (x + rect.X, y + rect.Y)
            right_pos = (rect.Width - 1 - x + rect.X, y + rect.Y)
            # Swap pixels
            pixels.swap(left_pos, right_pos)
```

### Rotate Left / Right (90°)
```python
def apply_rotate_90(pixels, action):
    rect = action.Rect
    temp_buffer = pixels.copy()
    clear_rect(pixels, rect)
    
    for y in range(rect.Height):
        for x in range(rect.Width):
            color = temp_buffer.get_pixel(x + rect.X, y + rect.Y)
            if color.a > 0:
                if action.Tool == RotateRight:
                    new_x = y
                    new_y = rect.Width - 1 - x
                else: # RotateLeft
                    new_x = rect.Height - 1 - y
                    new_y = x
                
                # Offset to center of selection
                dest_x = new_x + rect.X + (rect.Width - rect.Height) / 2
                dest_y = new_y + rect.Y + (rect.Height - rect.Width) / 2
                pixels.set_pixel_safe(dest_x, dest_y, color)
```

### RotateRect (Arbitrary Angle)

`Position[2]` is the new Top-Left. The rotated rect's
new top-left relative to its local coordinates is `(min_rx, min_ry)`
In global coordinates, it starts at `rect_min_x + position.X`.

The offset is essentially the difference from the original min
coordinates.

```python
def apply_rotate_rect(pixels, action):
    rect = ImageRect(action.Positions[0], action.Positions[1])
    pivot = action.Positions[2]
    angle_deg = action.Float
    
    # 1. Capture pixels in rect and clear them
    # 2. Use RotationContainer to rotate the buffer
    # 3. Draw rotated buffer at pivot position
```

### Color Adjustment
```python
def apply_color_adjustment(pixels, action):
    # Meta contains [hue_offset, saturation_multiplier, lightness_multiplier]
    adjustments = json.parse(action.Meta)
    
    if len(action.Positions) == 0:
        # Global adjustment
        TextureHelper.adjust(pixels, *adjustments)
    else:
        # Rect adjustment
        rect = ImageRect(action.Positions[0], action.Positions[1])
        for y in range(rect.Y, rect.Y + rect.Height):
            for x in range(rect.X, rect.X + rect.Width):
                p = pixels.get_pixel(x, y)
                pixels.set_pixel(x, y, TextureHelper.adjust_pixel(p, *adjustments))
```
