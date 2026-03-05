GraphDB C API

GraphDB C API 头文件
提供 GraphDB 的 C 语言接口

版本: 0.1.0
许可: Apache-2.0

更多信息请访问: https://github.com/kkkqkx123/graphDB

#ifndef GRAPHDB_H
#define GRAPHDB_H

#pragma once

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

/**
 * God角色的Space ID标记（全局角色，不绑定特定Space）
 */
#define GOD_SPACE_ID -1

#define DEFAULT_MAX_ALLOWED_CONNECTIONS 100

#define INDEX_KEY_SEPARATOR 255

/**
 * 等值查询默认选择性（假设10个不同值）
 */
#define EQUALITY 0.1

/**
 * 范围查询默认选择性（假设选择1/3的数据）
 */
#define RANGE 0.333

/**
 * 小于/大于查询默认选择性
 */
#define COMPARISON 0.333

/**
 * 不等查询默认选择性
 */
#define NOT_EQUAL 0.9

/**
 * IS NULL 查询选择性（通常很少为null）
 */
#define IS_NULL 0.05

/**
 * IS NOT NULL 查询选择性
 */
#define IS_NOT_NULL 0.95

/**
 * IN 查询默认选择性（假设3个值）
 */
#define IN_LIST 0.3

/**
 * EXISTS 查询选择性
 */
#define EXISTS 0.5

/**
 * 布尔AND操作的选择性惩罚
 */
#define AND_CORRELATION 0.9

/**
 * 布尔OR操作的选择性惩罚
 */
#define OR_CORRELATION 0.9

/**
 * 无效ID常量
 */
#define INVALID_ID -1

#endif  /* GRAPHDB_H */
