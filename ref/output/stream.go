package output

import (
	"errors"
	"os"
	"path/filepath"
)

// StreamOutput 输出流管理器
type StreamOutput struct {
	file   *os.File
	writer Writer
}

// InitStreamOutput 初始化输出流
// mode: console - 仅终端, file - 仅文件, both - 终端+文件
func InitStreamOutput(filePath, mode string, append bool) (*StreamOutput, error) {
	so := &StreamOutput{}

	switch mode {
	case "console", "":
		// 仅终端
		so.writer = os.Stdout
		return so, nil

	case "file":
		// 仅文件
		if filePath == "" {
			return nil, errors.New("output file path is required")
		}

		file, err := so.openFile(filePath, append)
		if err != nil {
			return nil, err
		}
		so.file = file
		so.writer = file
		return so, nil

	case "both":
		// 终端+文件
		if filePath == "" {
			return nil, errors.New("output file path is required")
		}

		file, err := so.openFile(filePath, append)
		if err != nil {
			return nil, err
		}
		so.file = file
		so.writer = NewMultiWriter(os.Stdout, file)
		return so, nil

	default:
		return nil, errors.New("invalid output mode: " + mode)
	}
}

// openFile 打开输出文件
func (so *StreamOutput) openFile(filePath string, append bool) (*os.File, error) {
	// 确保目录存在
	dir := filepath.Dir(filePath)
	if err := os.MkdirAll(dir, 0755); err != nil {
		return nil, err
	}

	// 打开文件
	if append {
		return os.OpenFile(filePath, os.O_CREATE|os.O_WRONLY|os.O_APPEND, 0644)
	}
	return os.Create(filePath)
}

// GetWriter 获取写入器
func (so *StreamOutput) GetWriter() Writer {
	return so.writer
}

// Close 关闭输出流
func (so *StreamOutput) Close() error {
	// 如果 writer 是 MultiWriter，调用其 Close 方法
	if mw, ok := so.writer.(*MultiWriter); ok {
		return mw.Close()
	}
	// 否则只关闭文件
	if so.file != nil {
		return so.file.Close()
	}
	return nil
}
