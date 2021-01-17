#include "kdmapper.hpp"
#include "utils.hpp"

extern "C" auto map_driver_from_memory(uint8_t *data, uint64_t len) -> uint32_t;
