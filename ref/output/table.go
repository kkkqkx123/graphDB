package output

import (
	"io"

	"github.com/olekukonko/tablewriter"
	"github.com/olekukonko/tablewriter/tw"
)

// Table 封装 tablewriter 的表格输出
type Table struct {
	table *tablewriter.Table
}

// NewTable 创建新的表格（使用默认 stdout）
func NewTable() *Table {
	return NewTableWithWriter(GetGlobalStdout())
}

// NewTableWithWriter 使用指定 Writer 创建表格
func NewTableWithWriter(w io.Writer) *Table {
	table := tablewriter.NewTable(w,
		tablewriter.WithHeaderAutoFormat(tw.On),
		tablewriter.WithRendition(tw.Rendition{
			Borders: tw.Border{Left: tw.Off, Right: tw.Off, Top: tw.Off, Bottom: tw.Off},
		}),
	)
	return &Table{table: table}
}

// NewTableWithOptions 创建带选项的表格（使用默认 stdout）
func NewTableWithOptions(opts ...tablewriter.Option) *Table {
	return NewTableWithOptionsWriter(GetGlobalStdout(), opts...)
}

// NewTableWithOptionsWriter 使用指定 Writer 和选项创建表格
func NewTableWithOptionsWriter(w io.Writer, opts ...tablewriter.Option) *Table {
	table := tablewriter.NewTable(w, opts...)
	return &Table{table: table}
}

// SetHeader 设置表头
func (t *Table) SetHeader(headers []string) {
	t.table.Header(headers)
}

// Append 添加一行数据
func (t *Table) Append(row []string) error {
	return t.table.Append(row)
}

// Render 渲染表格
func (t *Table) Render() error {
	return t.table.Render()
}

// PrintTable 快速打印表格（使用默认 stdout）
func PrintTable(headers []string, rows [][]string) error {
	return PrintTableWithWriter(GetGlobalStdout(), headers, rows)
}

// PrintTableWithWriter 使用指定 Writer 打印表格
func PrintTableWithWriter(w io.Writer, headers []string, rows [][]string) error {
	table := NewTableWithWriter(w)
	table.SetHeader(headers)
	for _, row := range rows {
		if err := table.Append(row); err != nil {
			return err
		}
	}
	return table.Render()
}

// PrintTableWithOptions 使用选项打印表格（使用默认 stdout）
func PrintTableWithOptions(headers []string, rows [][]string, opts ...tablewriter.Option) error {
	return PrintTableWithOptionsWriter(GetGlobalStdout(), headers, rows, opts...)
}

// PrintTableWithOptionsWriter 使用指定 Writer 和选项打印表格
func PrintTableWithOptionsWriter(w io.Writer, headers []string, rows [][]string, opts ...tablewriter.Option) error {
	table := NewTableWithOptionsWriter(w, opts...)
	table.SetHeader(headers)
	for _, row := range rows {
		if err := table.Append(row); err != nil {
			return err
		}
	}
	return table.Render()
}
