#include "lib.h"


int main(void) {
    char message[21] = "Hello from process ?\n";

    for (;;) {
        message[19] = pid() + 0x30;
        print(message, 22);
    }
    return -1;
}
