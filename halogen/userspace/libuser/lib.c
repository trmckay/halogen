#include "lib.h"

void __attribute__((naked, section(".text.init"))) _start(void) {
  __asm__("la sp, __sp\n"
          ".option push\n"
          ".option norelax\n"
          "la gp, __global_pointer$\n"
          "call main\n"
          "call exit\n");
}

i64 __attribute__((naked))
syscall(i64 code, i64 arg1, i64 arg2, i64 arg3, i64 arg4) {
  __asm__("ecall\n"
          "ret\n");
}

void print(const char *str, i64 len) {
  syscall(SYSCALL_PRINT, (i64)str, len, 0, 0);
}

void exit(i64 code) {
  syscall(SYSCALL_EXIT, code, 0, 0, 0);
  __asm__("unimp");
};
