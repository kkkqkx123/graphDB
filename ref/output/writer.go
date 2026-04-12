package output

import (
	"io"
	"os"
)

// Writer 定义输出写入器接口
type Writer interface {
	io.Writer
}

// Manager 输出管理器
type Manager struct {
	stdout Writer
	stderr Writer
	format string // 输出格式: table, json
}

// NewManager 创建新的输出管理器
func NewManager() *Manager {
	return &Manager{
		stdout: os.Stdout,
		stderr: os.Stderr,
		format: "table",
	}
}

// NewManagerWithWriter 使用指定 Writer 创建管理器
func NewManagerWithWriter(stdout, stderr Writer) *Manager {
	return &Manager{
		stdout: stdout,
		stderr: stderr,
		format: "table",
	}
}

// SetStdout 设置标准输出
func (m *Manager) SetStdout(w Writer) *Manager {
	m.stdout = w
	return m
}

// SetStderr 设置标准错误输出
func (m *Manager) SetStderr(w Writer) *Manager {
	m.stderr = w
	return m
}

// SetFormat 设置输出格式
func (m *Manager) SetFormat(format string) *Manager {
	m.format = format
	return m
}

// Stdout 获取标准输出
func (m *Manager) Stdout() Writer {
	return m.stdout
}

// Stderr 获取标准错误输出
func (m *Manager) Stderr() Writer {
	return m.stderr
}

// Format 获取输出格式
func (m *Manager) Format() string {
	return m.format
}

// IsJSONFormat 检查是否为 JSON 格式
func (m *Manager) IsJSONFormat() bool {
	return m.format == "json"
}

// IsTableFormat 检查是否为表格格式
func (m *Manager) IsTableFormat() bool {
	return m.format == "table"
}

// 全局默认管理器
var defaultManager = NewManager()

// DefaultManager 获取默认管理器
func DefaultManager() *Manager {
	return defaultManager
}

// SetDefaultManager 设置默认管理器
func SetDefaultManager(m *Manager) {
	defaultManager = m
}

// SetGlobalStdout 设置全局标准输出
func SetGlobalStdout(w Writer) {
	defaultManager.SetStdout(w)
}

// SetGlobalStderr 设置全局标准错误输出
func SetGlobalStderr(w Writer) {
	defaultManager.SetStderr(w)
}

// SetGlobalFormat 设置全局输出格式
func SetGlobalFormat(format string) {
	defaultManager.SetFormat(format)
}

// GetGlobalStdout 获取全局标准输出
func GetGlobalStdout() Writer {
	return defaultManager.Stdout()
}

// GetGlobalStderr 获取全局标准错误输出
func GetGlobalStderr() Writer {
	return defaultManager.Stderr()
}

// GetGlobalFormat 获取全局输出格式
func GetGlobalFormat() string {
	return defaultManager.Format()
}
