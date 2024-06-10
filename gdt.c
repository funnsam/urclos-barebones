#include <stdint.h>

const uint64_t GDT_ENTRIES[] = {
    0x0000000000000000,
    0x00af9a000000ffff,
    0x00af92000000ffff,
    // 0xffff00008092af00,
    // 0x0000000000000000, // tss
};

struct __attribute__((packed)) Gdtr {
    uint16_t volatile limit;
    uint32_t volatile base;
};

volatile struct Gdtr gdtr;

void set_gdt() {
    asm __volatile__ ("lgdt (gdtr)");
    return;
}

void reload_segments() {
    asm __volatile__ (" \
        ljmp $0x08, $.ok \n\
        .ok: \n\
        mov %0, %%ds \n\
        mov %0, %%es \n\
        mov %0, %%fs \n\
        mov %0, %%gs \n\
        mov %0, %%ss \n\
    "   :
        : "r" (0x10)
        :
    );
    return;
}

void init_gdt() {
    gdtr.limit = 3 * sizeof(uint64_t) - 1;
    gdtr.base = (uint32_t) GDT_ENTRIES;

    // set_gdt();
    // reload_segments();
}
