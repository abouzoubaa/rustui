#include <Block.h>

int invoke_block(int (^block)(int), int value) {
    return block(value);
}

void *make_block(int seed) {
    return Block_copy(^int(int value) { return seed + value; });
}
