# Bench

## System Structure

On boot, the `InitGopBufferLogger` driver is loaded. This driver does not require the heap allocator to be initialised. It is a standalone driver which does not rely on `DeviceManager`. After `memory::init()` is called, `DeviceManager` is called and initialises the IDT, and loads the APIC driver. The APIC driver initialises the ISA driver and the PCI driver which discover devices connected via their respective buses.

## Acknowledgement
This operating system uses the tutorials at https://os.phil-opp.com/ as a base.