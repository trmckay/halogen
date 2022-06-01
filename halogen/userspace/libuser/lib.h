#ifndef _HALOGEN_LIB_H
#define _HALOGEN_LIB_H


typedef unsigned char u8;
typedef char i8;

typedef unsigned short u16;
typedef short i16;

typedef unsigned int u32;
typedef int i32;

typedef long int i64;
typedef unsigned long int u64;

typedef long long int i128;
typedef unsigned long long int u128;

#define SYSCALL_EXIT 0
#define SYSCALL_PRINT 1

void print(const char *str, i64 len);
void exit(i64 code);

#endif
