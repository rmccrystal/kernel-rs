#define _AMD64_

#include <ntdef.h>
#include <ntifs.h>
#include <ntddk.h>
#include <windef.h>
#include <ntstrsafe.h>

NTKERNELAPI
NTSTATUS
NTAPI
MmCopyVirtualMemory(
        _In_ PEPROCESS FromProcess,
        _In_ PVOID FromAddress,
        _In_ PEPROCESS ToProcess,
        _Out_ PVOID ToAddress,
        _In_ SIZE_T BufferSize,
        _In_ KPROCESSOR_MODE PreviousMode,
        _Out_ PSIZE_T NumberOfBytesCopied
);