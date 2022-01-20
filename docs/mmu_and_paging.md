---
title: MMU and paging (Sv39)
---

# Sv39

* Sv39 translates 39-bit virtual addresses to 56-bit virtual-addresses
* Leaves can occur at any level
* A single PTE is 8 bytes (64 bits)

# SATP register

The `SATP` register controls paging.

- `SATP[63:60] = mode`: paging mode, bare/Sv39
- `SATP[59:44] = ASID`: address-space identifier
- `SATP[43:00] = PPN`: top 44 bits of physical page number

# Address translation

Address translation can either be done by the MMU or manually by the kernel.

1. Read the SATP register to get the PPN; shift left 12 to create 56-bit physical address
2. Add `VPN_2 << 3` (multiply by 8 since that is the size of a PTE)
3. Dereference this address to get the L2 page-table entry
4. If the valid bit is set continue, otherwise that's a page-fault
5. If `XWR == 0`, this is a branch
6. If `XWR != 0`, this is a leaf

## Leaves at each level

If a leaf is found immediately at level 2, `VPN_1` and `VPN_0` do not get translated (they
would otherwise be used in the L1 and L0 page tables). The whole virtual address—other than
`PPN_2`, which comes from the L2 page-table—is preserved in the physical address.

For a leaf at level one, `VPN_2` and `VPN_1` are translated, while `VPN_0` and the offset
are untranslated.

In summary:

- L2 leaf:
- + `phys_addr = { (L2_PT[VPN_2 * 8])[38:30], VPN_1, VPN_0, offset }`

- L1 leaf:
- + `L1_PT = L2_PT[VPN_2 * 8]`
- + `phys_addr = { (L1_PT[VPN_1 * 8])[38:21], VPN_0, offset }`

- L0 leaf:
  + `L1_PT = L2_PT[VPN_2 * 8]`
  + `L0_PT = L1_PT[VPN_1 * 8]`
  + `phys_addr = { (L0_PT[VPN_0 * 8])[38:12], offset }`
