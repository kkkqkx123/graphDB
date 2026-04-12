package output

import (
	"io"
	"os"
	"sync"
)

// MultiWriter 多流写入器
type MultiWriter struct {
	writers []io.Writer
	mu      sync.Mutex
}

// NewMultiWriter 创建多流写入器
func NewMultiWriter(writers ...io.Writer) *MultiWriter {
	return &MultiWriter{
		writers: writers,
	}
}

// Write 实现 io.Writer 接口
func (mw *MultiWriter) Write(p []byte) (n int, err error) {
	mw.mu.Lock()
	defer mw.mu.Unlock()

	for _, w := range mw.writers {
		n, err = w.Write(p)
		if err != nil {
			return
		}
	}
	return len(p), nil
}

// AddWriter 添加写入器
func (mw *MultiWriter) AddWriter(w io.Writer) {
	mw.mu.Lock()
	defer mw.mu.Unlock()
	mw.writers = append(mw.writers, w)
}

// Close 关闭所有可关闭的写入器
func (mw *MultiWriter) Close() error {
	mw.mu.Lock()
	defer mw.mu.Unlock()

	for _, w := range mw.writers {
		if closer, ok := w.(io.Closer); ok && w != os.Stdout && w != os.Stderr {
			if err := closer.Close(); err != nil {
				return err
			}
		}
	}
	return nil
}
