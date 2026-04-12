package output

import (
	"fmt"
	"io"

	"github.com/fatih/color"
)

// Color 颜色输出管理器
type Color struct {
	success *color.Color
	error   *color.Color
	warning *color.Color
	info    *color.Color
}

// NewColor 创建新的颜色管理器（不绑定 writer）
func NewColor() *Color {
	return &Color{
		success: color.New(color.FgGreen),
		error:   color.New(color.FgRed),
		warning: color.New(color.FgYellow),
		info:    color.New(color.FgCyan),
	}
}

// Success 打印成功信息
func (c *Color) Success(format string, a ...interface{}) {
	fmt.Fprintf(GetGlobalStdout(), "%s\n", c.success.Sprintf("✓ "+format, a...))
}

// Error 打印错误信息
func (c *Color) Error(format string, a ...interface{}) {
	fmt.Fprintf(GetGlobalStderr(), "%s\n", c.error.Sprintf("✗ "+format, a...))
}

// Warning 打印警告信息
func (c *Color) Warning(format string, a ...interface{}) {
	fmt.Fprintf(GetGlobalStdout(), "%s\n", c.warning.Sprintf("⚠ "+format, a...))
}

// Info 打印信息
func (c *Color) Info(format string, a ...interface{}) {
	fmt.Fprintf(GetGlobalStdout(), "%s\n", c.info.Sprintf("ℹ "+format, a...))
}

// SuccessString 返回成功格式的字符串
func (c *Color) SuccessString(format string, a ...interface{}) string {
	return c.success.Sprintf("✓ "+format, a...)
}

// ErrorString 返回错误格式的字符串
func (c *Color) ErrorString(format string, a ...interface{}) string {
	return c.error.Sprintf("✗ "+format, a...)
}

// WarningString 返回警告格式的字符串
func (c *Color) WarningString(format string, a ...interface{}) string {
	return c.warning.Sprintf("⚠ "+format, a...)
}

// InfoString 返回信息格式的字符串
func (c *Color) InfoString(format string, a ...interface{}) string {
	return c.info.Sprintf("ℹ "+format, a...)
}

// 全局颜色管理器实例（只读，存储颜色配置）
var globalColor = NewColor()

// Success 打印成功信息（使用默认 stdout）
func Success(format string, a ...interface{}) {
	fmt.Fprintf(GetGlobalStdout(), "%s\n", globalColor.success.Sprintf("✓ "+format, a...))
}

// Error 打印错误信息到 stderr（使用默认 stderr）
func Error(format string, a ...interface{}) {
	fmt.Fprintf(GetGlobalStderr(), "%s\n", globalColor.error.Sprintf("✗ "+format, a...))
}

// Warning 打印警告信息（使用默认 stdout）
func Warning(format string, a ...interface{}) {
	fmt.Fprintf(GetGlobalStdout(), "%s\n", globalColor.warning.Sprintf("⚠ "+format, a...))
}

// Warningf 打印警告信息（别名，与 Warning 相同）
func Warningf(format string, a ...interface{}) {
	Warning(format, a...)
}

// Info 打印信息（使用默认 stdout）
func Info(format string, a ...interface{}) {
	fmt.Fprintf(GetGlobalStdout(), "%s\n", globalColor.info.Sprintf("ℹ "+format, a...))
}

// SuccessString 返回成功格式的字符串
func SuccessString(format string, a ...interface{}) string {
	return globalColor.SuccessString(format, a...)
}

// ErrorString 返回错误格式的字符串
func ErrorString(format string, a ...interface{}) string {
	return globalColor.ErrorString(format, a...)
}

// WarningString 返回警告格式的字符串
func WarningString(format string, a ...interface{}) string {
	return globalColor.WarningString(format, a...)
}

// InfoString 返回信息格式的字符串
func InfoString(format string, a ...interface{}) string {
	return globalColor.InfoString(format, a...)
}

// Green 打印绿色文本（使用默认 stdout）
func Green(format string, a ...interface{}) {
	fmt.Fprintf(GetGlobalStdout(), "%s\n", color.GreenString(format, a...))
}

// Red 打印红色文本到 stderr（使用默认 stderr）
func Red(format string, a ...interface{}) {
	fmt.Fprintf(GetGlobalStderr(), "%s\n", color.RedString(format, a...))
}

// Yellow 打印黄色文本（使用默认 stdout）
func Yellow(format string, a ...interface{}) {
	fmt.Fprintf(GetGlobalStdout(), "%s\n", color.YellowString(format, a...))
}

// Cyan 打印青色文本（使用默认 stdout）
func Cyan(format string, a ...interface{}) {
	fmt.Fprintf(GetGlobalStdout(), "%s\n", color.CyanString(format, a...))
}

// GreenString 返回绿色字符串
func GreenString(format string, a ...interface{}) string {
	return color.GreenString(format, a...)
}

// RedString 返回红色字符串
func RedString(format string, a ...interface{}) string {
	return color.RedString(format, a...)
}

// YellowString 返回黄色字符串
func YellowString(format string, a ...interface{}) string {
	return color.YellowString(format, a...)
}

// CyanString 返回青色字符串
func CyanString(format string, a ...interface{}) string {
	return color.CyanString(format, a...)
}

// SetNoColor 禁用颜色输出
func SetNoColor(noColor bool) {
	color.NoColor = noColor
}

// IsColorEnabled 检查颜色是否启用
func IsColorEnabled() bool {
	return !color.NoColor
}

// PrintColored 根据类型打印彩色文本（使用默认 stdout）
func PrintColored(msgType, message string) {
	PrintColoredWithWriter(GetGlobalStdout(), msgType, message)
}

// PrintColoredWithWriter 使用指定 Writer 根据类型打印彩色文本
func PrintColoredWithWriter(w io.Writer, msgType, message string) {
	switch msgType {
	case "success":
		c := color.New(color.FgGreen)
		c.Fprint(w, c.Sprintf("✓ %s\n", message))
	case "error":
		c := color.New(color.FgRed)
		c.Fprint(w, c.Sprintf("✗ %s\n", message))
	case "warning":
		c := color.New(color.FgYellow)
		c.Fprint(w, c.Sprintf("⚠ %s\n", message))
	case "info":
		c := color.New(color.FgCyan)
		c.Fprint(w, c.Sprintf("ℹ %s\n", message))
	default:
		fmt.Fprintf(w, "%s\n", message)
	}
}

// FSuccess 使用指定 Writer 打印成功信息
func FSuccess(w io.Writer, format string, a ...interface{}) {
	c := color.New(color.FgGreen)
	c.Fprint(w, c.Sprintf("✓ "+format+"\n", a...))
}

// FError 使用指定 Writer 打印错误信息
func FError(w io.Writer, format string, a ...interface{}) {
	c := color.New(color.FgRed)
	c.Fprint(w, c.Sprintf("✗ "+format+"\n", a...))
}

// FWarning 使用指定 Writer 打印警告信息
func FWarning(w io.Writer, format string, a ...interface{}) {
	c := color.New(color.FgYellow)
	c.Fprint(w, c.Sprintf("⚠ "+format+"\n", a...))
}

// FInfo 使用指定 Writer 打印信息
func FInfo(w io.Writer, format string, a ...interface{}) {
	c := color.New(color.FgCyan)
	c.Fprint(w, c.Sprintf("ℹ "+format+"\n", a...))
}

// StatusOK 返回 OK 状态的彩色字符串
func StatusOK() string {
	return GreenString("✓")
}

// StatusError 返回 Error 状态的彩色字符串
func StatusError() string {
	return RedString("✗")
}

// StatusWarning 返回 Warning 状态的彩色字符串
func StatusWarning() string {
	return YellowString("⚠")
}

// StatusUnknown 返回 Unknown 状态的彩色字符串
func StatusUnknown() string {
	return YellowString("?")
}

// StatusString 根据布尔值返回状态彩色字符串
func StatusString(ok bool) string {
	if ok {
		return StatusOK()
	}
	return StatusError()
}

// StatusAlive 返回存活状态的彩色字符串
func StatusAlive(alive bool) string {
	if alive {
		return GreenString("可用")
	}
	return RedString("不可用")
}

// DelayStatus 根据延迟值返回延迟状态的颜色字符串
func DelayStatus(delay uint16) string {
	if delay == 0 {
		return YellowString("超时")
	}
	if delay < 100 {
		return GreenString("优秀")
	}
	if delay < 300 {
		return YellowString("良好")
	}
	return RedString("较慢")
}

// DelayColor 根据延迟值返回颜色字符串
func DelayColor(delay uint16) string {
	if delay == 0 {
		return YellowString(fmt.Sprintf("%dms", delay))
	}
	if delay < 100 {
		return GreenString(fmt.Sprintf("%dms", delay))
	}
	if delay < 300 {
		return YellowString(fmt.Sprintf("%dms", delay))
	}
	return RedString(fmt.Sprintf("%dms", delay))
}

// BoolString 返回布尔值的彩色字符串
func BoolString(b bool, trueStr, falseStr string) string {
	if b {
		return GreenString(trueStr)
	}
	return RedString(falseStr)
}

// BoolStringSimple 返回布尔值的彩色字符串（使用"启用"/"禁用"）
func BoolStringSimple(b bool) string {
	return BoolString(b, "启用", "禁用")
}

// Highlight 高亮文本（黄色背景）
func Highlight(format string, a ...interface{}) string {
	return color.HiYellowString(format, a...)
}

// HighlightWithWriter 使用指定 Writer 高亮文本
func HighlightWithWriter(w io.Writer, format string, a ...interface{}) {
	c := color.New(color.BgYellow, color.FgBlack)
	c.Fprint(w, c.Sprintf(" "+format+" \n", a...))
}

// PrintHighlight 打印高亮文本
func PrintHighlight(format string, a ...interface{}) {
	HighlightWithWriter(GetGlobalStdout(), format, a...)
}

// Dim 返回暗淡文本
func Dim(format string, a ...interface{}) string {
	return color.HiBlackString(format, a...)
}

// DimWithWriter 使用指定 Writer 打印暗淡文本
func DimWithWriter(w io.Writer, format string, a ...interface{}) {
	fmt.Fprint(w, Dim(format+"\n", a...))
}

// PrintDim 打印暗淡文本
func PrintDim(format string, a ...interface{}) {
	DimWithWriter(GetGlobalStdout(), format, a...)
}
