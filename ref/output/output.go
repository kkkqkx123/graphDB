package output

import (
	"fmt"
	"io"
	"strings"
)

// Print 打印输出（根据格式，使用默认 stdout）
func Print(data interface{}, format string) error {
	return PrintWithWriter(GetGlobalStdout(), data, format)
}

// PrintWithWriter 使用指定 Writer 打印输出
func PrintWithWriter(w io.Writer, data interface{}, format string) error {
	if format == "json" {
		return PrintJSONWithWriter(w, data)
	}
	_, err := fmt.Fprintf(w, "%v\n", data)
	return err
}

// PrintError 打印错误信息（使用默认 stderr）
func PrintError(msg string) error {
	return PrintErrorWithWriter(GetGlobalStderr(), msg)
}

// PrintErrorWithWriter 使用指定 Writer 打印错误信息
func PrintErrorWithWriter(w io.Writer, msg string) error {
	FError(w, msg)
	return nil
}

// PrintSuccess 打印成功信息（使用默认 stdout）
func PrintSuccess(msg string) {
	Success(msg)
}

// PrintSuccessWithWriter 使用指定 Writer 打印成功信息
func PrintSuccessWithWriter(w io.Writer, msg string) {
	FSuccess(w, msg)
}

// PrintWarning 打印警告信息（使用默认 stdout）
func PrintWarning(msg string) {
	Warning(msg)
}

// PrintWarningWithWriter 使用指定 Writer 打印警告信息
func PrintWarningWithWriter(w io.Writer, msg string) {
	FWarning(w, msg)
}

// PrintInfo 打印信息（使用默认 stdout）
func PrintInfo(msg string) {
	Info(msg)
}

// PrintInfoWithWriter 使用指定 Writer 打印信息
func PrintInfoWithWriter(w io.Writer, msg string) {
	FInfo(w, msg)
}

// Println 打印一行（使用默认 stdout）
func Println(a ...interface{}) {
	fmt.Fprintln(GetGlobalStdout(), a...)
}

// PrintRaw 打印（使用默认 stdout，不换行）
func PrintRaw(a ...interface{}) {
	fmt.Fprint(GetGlobalStdout(), a...)
}

// Printf 格式化打印（使用默认 stdout）
func Printf(format string, a ...interface{}) {
	fmt.Fprintf(GetGlobalStdout(), format, a...)
}

// PrintlnWithWriter 使用指定 Writer 打印一行
func PrintlnWithWriter(w io.Writer, a ...interface{}) {
	fmt.Fprintln(w, a...)
}

// PrintRawWithWriter 使用指定 Writer 打印（不换行）
func PrintRawWithWriter(w io.Writer, a ...interface{}) {
	fmt.Fprint(w, a...)
}

// PrintfWithWriter 使用指定 Writer 格式化打印
func PrintfWithWriter(w io.Writer, format string, a ...interface{}) {
	fmt.Fprintf(w, format, a...)
}

// PrintIndent 打印缩进文本
func PrintIndent(level int, a ...interface{}) {
	indent := strings.Repeat("  ", level)
	fmt.Fprint(GetGlobalStdout(), indent)
	fmt.Fprintln(GetGlobalStdout(), a...)
}

// PrintIndentWithWriter 使用指定 Writer 打印缩进文本
func PrintIndentWithWriter(w io.Writer, level int, a ...interface{}) {
	indent := strings.Repeat("  ", level)
	fmt.Fprint(w, indent)
	fmt.Fprintln(w, a...)
}

// PrintSeparator 打印分隔线
func PrintSeparator(char string, length int) {
	if char == "" {
		char = "-"
	}
	if length <= 0 {
		length = 80
	}
	fmt.Fprintln(GetGlobalStdout(), strings.Repeat(char, length))
}

// PrintSeparatorWithWriter 使用指定 Writer 打印分隔线
func PrintSeparatorWithWriter(w io.Writer, char string, length int) {
	if char == "" {
		char = "-"
	}
	if length <= 0 {
		length = 80
	}
	fmt.Fprintln(w, strings.Repeat(char, length))
}

// PrintList 打印列表
func PrintList(title string, items []string) {
	if title != "" {
		fmt.Fprintf(GetGlobalStdout(), "%s:\n", title)
	}
	for _, item := range items {
		fmt.Fprintf(GetGlobalStdout(), "  - %s\n", item)
	}
}

// PrintListWithWriter 使用指定 Writer 打印列表
func PrintListWithWriter(w io.Writer, title string, items []string) {
	if title != "" {
		fmt.Fprintf(w, "%s:\n", title)
	}
	for _, item := range items {
		fmt.Fprintf(w, "  - %s\n", item)
	}
}

// PrintKeyValue 打印键值对
func PrintKeyValue(key string, value interface{}) {
	fmt.Fprintf(GetGlobalStdout(), "  %s: %v\n", key, value)
}

// PrintKeyValueWithWriter 使用指定 Writer 打印键值对
func PrintKeyValueWithWriter(w io.Writer, key string, value interface{}) {
	fmt.Fprintf(w, "  %s: %v\n", key, value)
}

// PrintKeyValueBlock 打印键值对块
func PrintKeyValueBlock(title string, pairs map[string]interface{}) {
	if title != "" {
		fmt.Fprintf(GetGlobalStdout(), "%s:\n", title)
	}
	for key, value := range pairs {
		PrintKeyValue(key, value)
	}
}

// PrintKeyValueBlockWithWriter 使用指定 Writer 打印键值对块
func PrintKeyValueBlockWithWriter(w io.Writer, title string, pairs map[string]interface{}) {
	if title != "" {
		fmt.Fprintf(w, "%s:\n", title)
	}
	for key, value := range pairs {
		PrintKeyValueWithWriter(w, key, value)
	}
}

// PrintSection 打印区块标题
func PrintSection(title string) {
	fmt.Fprintf(GetGlobalStdout(), "\n%s\n", title)
}

// PrintSectionWithWriter 使用指定 Writer 打印区块标题
func PrintSectionWithWriter(w io.Writer, title string) {
	fmt.Fprintf(w, "\n%s\n", title)
}

// PrintEmptyLine 打印空行
func PrintEmptyLine() {
	fmt.Fprintln(GetGlobalStdout())
}

// PrintEmptyLineWithWriter 使用指定 Writer 打印空行
func PrintEmptyLineWithWriter(w io.Writer) {
	fmt.Fprintln(w)
}
