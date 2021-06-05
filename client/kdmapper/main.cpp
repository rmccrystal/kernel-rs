#include "kdmapper.hpp"

HANDLE iqvw64e_device_handle;

LONG WINAPI SimplestCrashHandler(EXCEPTION_POINTERS* ExceptionInfo)
{
	
	std::cout << "[!!] Crash at addr 0x" << ExceptionInfo->ExceptionRecord->ExceptionAddress << " by 0x" << std::hex << ExceptionInfo->ExceptionRecord->ExceptionCode << std::endl;
	
	if (iqvw64e_device_handle)
		intel_driver::Unload(iqvw64e_device_handle);

	return EXCEPTION_EXECUTE_HANDLER;
}

int main(const int argc, char** argv)
{
	SetUnhandledExceptionFilter(SimplestCrashHandler);

	srand((unsigned)time(NULL) * GetCurrentThreadId());
	if (argc != 2 || std::filesystem::path(argv[1]).extension().string().compare(".sys"))
	{
		std::cout << "[-] Incorrect usage" << std::endl;
		return -1;
	}

	//from https://github.com/ShoaShekelbergstein/kdmapper as some Drivers takes same device name
	if (intel_driver::IsRunning())
	{
		std::cout << "[-] \\Device\\Nal is already in use." << std::endl;
		return -1;
	}

	const std::string driver_path = argv[1];

	if (!std::filesystem::exists(driver_path))
	{
		std::cout << "[-] File " << driver_path << " doesn't exist" << std::endl;
		return -1;
	}

	iqvw64e_device_handle = intel_driver::Load();

	if (!iqvw64e_device_handle || iqvw64e_device_handle == INVALID_HANDLE_VALUE)
	{
		std::cout << "[-] Failed to load driver iqvw64e.sys" << std::endl;
		intel_driver::Unload(iqvw64e_device_handle);
		return -1;
	}

	if (!intel_driver::ClearPiDDBCacheTable(iqvw64e_device_handle)) {
		std::cout << "[-] Failed to ClearPiDDBCacheTable" << std::endl;
		intel_driver::Unload(iqvw64e_device_handle);
		return -1;
	}

	if (!intel_driver::ClearKernelHashBucketList(iqvw64e_device_handle)) {
		std::cout << "[-] Failed to ClearKernelHashBucketList" << std::endl;
		intel_driver::Unload(iqvw64e_device_handle);
		return -1;
	}

	if (!intel_driver::ClearMmUnloadedDrivers(iqvw64e_device_handle)) {
		std::cout << "[!] Failed to ClearMmUnloadedDrivers" << std::endl;
		intel_driver::Unload(iqvw64e_device_handle);
		return -1;
	}

	if (!kdmapper::MapDriver(iqvw64e_device_handle, driver_path))
	{
		std::cout << "[-] Failed to map " << driver_path << std::endl;
		intel_driver::Unload(iqvw64e_device_handle);
		return -1;
	}

	intel_driver::Unload(iqvw64e_device_handle);
	std::cout << "[+] success" << std::endl;

}