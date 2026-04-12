package output

import (
	"encoding/json"
	"io"

	pkgerrors "github.com/kkkqkx123/mihomo-cli/pkg/errors"
)

// JSONEncoder JSON 编码器配置
type JSONEncoder struct {
	writer     io.Writer
	indent     string
	prefix     string
	escapeHTML bool
}

// NewJSONEncoder 创建新的 JSON 编码器（使用默认 stdout）
func NewJSONEncoder() *JSONEncoder {
	return NewJSONEncoderWithWriter(GetGlobalStdout())
}

// NewJSONEncoderWithWriter 使用指定 Writer 创建 JSON 编码器
func NewJSONEncoderWithWriter(w io.Writer) *JSONEncoder {
	return &JSONEncoder{
		writer:     w,
		indent:     "  ",
		prefix:     "",
		escapeHTML: true,
	}
}

// SetWriter 设置输出 Writer
func (e *JSONEncoder) SetWriter(w io.Writer) *JSONEncoder {
	e.writer = w
	return e
}

// SetIndent 设置缩进
func (e *JSONEncoder) SetIndent(prefix, indent string) *JSONEncoder {
	e.prefix = prefix
	e.indent = indent
	return e
}

// SetEscapeHTML 设置是否转义 HTML
func (e *JSONEncoder) SetEscapeHTML(escape bool) *JSONEncoder {
	e.escapeHTML = escape
	return e
}

// Encode 编码并输出
func (e *JSONEncoder) Encode(data interface{}) error {
	if e.writer == nil {
		return pkgerrors.ErrInvalidArg("writer is nil", nil)
	}
	encoder := json.NewEncoder(e.writer)
	encoder.SetIndent(e.prefix, e.indent)
	encoder.SetEscapeHTML(e.escapeHTML)
	return encoder.Encode(data)
}

// EncodeToString 编码为字符串
func (e *JSONEncoder) EncodeToString(data interface{}) (string, error) {
	var result []byte
	var err error

	if e.indent != "" {
		result, err = json.MarshalIndent(data, e.prefix, e.indent)
	} else {
		result, err = json.Marshal(data)
	}

	if err != nil {
		return "", err
	}
	return string(result), nil
}

// PrintJSON 以美化格式打印 JSON（使用默认 stdout）
func PrintJSON(data interface{}) error {
	return PrintJSONWithWriter(GetGlobalStdout(), data)
}

// PrintJSONWithWriter 使用指定 Writer 打印 JSON
func PrintJSONWithWriter(w io.Writer, data interface{}) error {
	encoder := NewJSONEncoderWithWriter(w)
	return encoder.Encode(data)
}

// PrintJSONCompact 以紧凑格式打印 JSON（使用默认 stdout）
func PrintJSONCompact(data interface{}) error {
	return PrintJSONCompactWithWriter(GetGlobalStdout(), data)
}

// PrintJSONCompactWithWriter 使用指定 Writer 打印紧凑 JSON
func PrintJSONCompactWithWriter(w io.Writer, data interface{}) error {
	encoder := NewJSONEncoderWithWriter(w).SetIndent("", "")
	return encoder.Encode(data)
}

// ToJSONString 将数据转换为 JSON 字符串
func ToJSONString(data interface{}) (string, error) {
	return NewJSONEncoder().EncodeToString(data)
}

// ToJSONStringCompact 将数据转换为紧凑 JSON 字符串
func ToJSONStringCompact(data interface{}) (string, error) {
	return NewJSONEncoder().SetIndent("", "").EncodeToString(data)
}

// PrintFormattedJSON 根据格式参数输出 JSON（使用默认 stdout）
func PrintFormattedJSON(data interface{}, compact bool) error {
	return PrintFormattedJSONWithWriter(GetGlobalStdout(), data, compact)
}

// PrintFormattedJSONWithWriter 使用指定 Writer 根据格式参数输出 JSON
func PrintFormattedJSONWithWriter(w io.Writer, data interface{}, compact bool) error {
	if compact {
		return PrintJSONCompactWithWriter(w, data)
	}
	return PrintJSONWithWriter(w, data)
}

// PrintJSONWithPrefix 以指定前缀打印 JSON（使用默认 stdout）
func PrintJSONWithPrefix(data interface{}, prefix string) error {
	return PrintJSONWithPrefixWriter(GetGlobalStdout(), data, prefix)
}

// PrintJSONWithPrefixWriter 使用指定 Writer 以指定前缀打印 JSON
func PrintJSONWithPrefixWriter(w io.Writer, data interface{}, prefix string) error {
	encoder := NewJSONEncoderWithWriter(w).SetIndent(prefix, "  ")
	return encoder.Encode(data)
}

// FormatJSON 格式化 JSON 字符串
func FormatJSON(jsonStr string) (string, error) {
	var data interface{}
	if err := json.Unmarshal([]byte(jsonStr), &data); err != nil {
		return "", pkgerrors.ErrInvalidArg("invalid JSON", err)
	}
	return ToJSONString(data)
}
