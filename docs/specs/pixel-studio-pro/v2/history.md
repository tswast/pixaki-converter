# History & Actions

The `_historyJson` field in a [Layer](layer.md) contains the actual drawing data and a sequence of tool actions.

## History Structure

| Property | Type | Description |
| :--- | :--- | :--- |
| `Actions` | Array | A sequence of [Action](#action-object) objects. |
| `Index` | Integer | The current position in the action history. |
| `_source` | String | (Optional) A Base64 encoded PNG of the flattened layer content. |

## Action Object

An individual drawing command.

| Property | Type | Description |
| :--- | :--- | :--- |
| `Tool` | Integer | The tool ID used (e.g., `0`: Pen, `1`: Eraser, `2`: Selection, `3`: Bucket, `6`: Line/Shape, `10`: Move, `20`: Paste/Import). |
| `ColorIndexes` | Array | (Optional) Indexes of colors used. |
| `Positions` | String | Base64 encoded coordinate data. |
| `Colors` | String | Base64 encoded color data (usually RGBA). |
| `Meta` | String | (Optional) A JSON string containing tool-specific metadata (e.g., `Rect`, `Pixels`). |
| `Invalid` | Boolean | Whether the action is considered invalid. |

### Tool Details

Different tools utilize the properties of the Action object in specific ways:

* **Tool 3 (Paint Bucket)**:
  * `Positions`: Contains a single `(X, Y)` coordinate representing the starting point of the fill.
  * `Colors`: Contains a single 4-byte RGBA value representing the fill color.
* **Tool 6 (Line/Shape)**:
  * `Positions`: Contains an array of `(X, Y)` coordinate pairs for all the pixels that make up the shape or line.
* **Tool 10 (Move)**:
  * `Positions`: Contains two `(X, Y)` coordinate pairs, likely representing the start and end points of the movement vector.
  * `Meta`: Contains a JSON string with a rectangle defining the selection bounds, formatted as `{"From":{"X":...,"Y":...},"To":{"X":...,"Y":...}}`.
* **Tool 20 (Paste / Import)**:
  * `Meta`: Contains a JSON string detailing the pasted image:
    * `Rect`: The destination rectangle `{"From": {"X", "Y"}, "To": {"X", "Y"}}`.
    * `RectSource`: The source rectangle in the original image.
    * `Pixels`: Base64 encoded PNG image data of the specific pasted region.

### Encoding
`Positions` and `Colors` use a custom Base64 binary encoding to store coordinate and color arrays efficiently. `Positions` encodes a sequence of 16-bit little-endian unsigned integers, forming `(X, Y)` coordinate pairs.
