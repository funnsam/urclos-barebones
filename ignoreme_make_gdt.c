#include <stdio.h>
#include<stdint.h>
struct GDT {
    int base;
    int limit;
    int access_byte;
    int flags;
};

void encodeGdtEntry(uint8_t *target, struct GDT source) {
    // Encode the limit
    target[0] = source.limit & 0xFF;
    target[1] = (source.limit >> 8) & 0xFF;
    target[6] = (source.limit >> 16) & 0x0F;

    // Encode the base
    target[2] = source.base & 0xFF;
    target[3] = (source.base >> 8) & 0xFF;
    target[4] = (source.base >> 16) & 0xFF;
    target[7] = (source.base >> 24) & 0xFF;

    // Encode the access byte
    target[5] = source.access_byte;

    // Encode the flags
    target[6] |= (source.flags << 4);
}

int main() {
    uint8_t t[8] = {0};
    struct GDT gdt;
    gdt.base = 0x00800000;
    gdt.limit = 0x003FFFFF;
    gdt.access_byte = 0x92;
    gdt.flags = 0x9A;
    encodeGdtEntry(t, gdt);
    printf("%02x%02x%02x%02x%02x%02x%02x%02x\n", t[0], t[1], t[2], t[3], t[4], t[5], t[6], t[7]);
    return 0;
}
