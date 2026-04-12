package output

import (
	"github.com/kkkqkx123/mihomo-cli/pkg/errors"
)

// OutputConfig 输出配置
type OutputConfig struct {
	File   string `mapstructure:"file"`   // 输出文件路径
	Mode   string `mapstructure:"mode"`   // 输出模式: console/file/both
	Append bool   `mapstructure:"append"` // 是否追加模式
}

// Validate 验证配置
func (c *OutputConfig) Validate() error {
	if c.Mode != "" && c.Mode != "console" && c.Mode != "file" && c.Mode != "both" {
		return errors.ErrConfig("output mode must be console, file, or both", nil)
	}
	if (c.Mode == "file" || c.Mode == "both") && c.File == "" {
		return errors.ErrConfig("output file path is required when mode is file or both", nil)
	}
	return nil
}
