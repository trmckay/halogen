#include "lib.h"

static const char *message = "Hello from userspace!\n";

int main(void) {
    print(message, 22);
    return -1;
}
