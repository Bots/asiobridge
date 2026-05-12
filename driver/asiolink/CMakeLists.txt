cmake_minimum_required(VERSION 3.20)
project(asiobridge_asio VERSION 1.0.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# ASIO SDK path - requires Steinberg ASIO SDK
set(ASIO_SDK_PATH "${CMAKE_CURRENT_SOURCE_DIR}/../../vendor/asio-sdk" CACHE PATH "Path to Steinberg ASIO SDK")

# Include ASIO SDK headers
include_directories(
  "${ASIO_SDK_PATH}/host"
  "${ASIO_SDK_PATH}/pluginterfaces"
)

# Source files
set(SOURCES
  asiobridge_asio.cpp
)

# Create DLL
add_library(asiobridge_asio SHARED ${SOURCES})

# Windows-specific settings
if(WIN32)
  target_compile_definitions(asiobridge_asio PRIVATE
    ASIO_NO_EXCEPTIONS
    _WINDOWS
    DLLAPI=__declspec(dllexport)
  )

  target_link_libraries(asiobridge_asio
    kernel32.lib
    user32.lib
    gdi32.lib
    ole32.lib
  )

  # Set DLL output directory
  set_target_properties(asiobridge_asio PROPERTIES
    RUNTIME_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/$<CONFIG>"
    LIBRARY_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/$<CONFIG>"
    ARCHIVE_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/$<CONFIG>"
  )
endif()

# Install
install(TARGETS asiobridge_asio
  LIBRARY DESTINATION lib
  RUNTIME DESTINATION bin
  ARCHIVE DESTINATION lib
)
