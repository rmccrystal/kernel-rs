#include "exports.h"

extern "C" auto map_driver_from_memory(uint8_t *data, uint64_t len) -> uint32_t {
    std::vector<uint8_t> driver(data, data + len);

    HANDLE iqvw64e_device_handle = intel_driver::Load();

    if (!iqvw64e_device_handle || iqvw64e_device_handle == INVALID_HANDLE_VALUE)
    {
        return 1; // Failed to load driver
    }

    if (!kdmapper::MapDriver(iqvw64e_device_handle, driver))
    {
        return 2; // Failed to map;
        intel_driver::Unload(iqvw64e_device_handle);
    }

    intel_driver::Unload(iqvw64e_device_handle);
}