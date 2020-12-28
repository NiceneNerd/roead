#include "roead/include/sarc.h"
#include "roead/src/lib.rs.h"
#include "rust/cxx.h"
#include <memory>
#include <iostream>
#include <stdexcept>
#include <string_view>
#include <nonstd/span.h>
#include <oead/sarc.h>

SarcFile::SarcFile(const oead::Sarc::File file) : inner(std::move(file)) {}
rust::Str SarcFile::name() const {
    return rust::Str(inner.name.data(), inner.name.size());
}

rust::Slice<const uint8_t> SarcFile::data() const {
    return rust::Slice<const uint8_t>(inner.data.data(), inner.data.size());
}

Sarc::Sarc(const rust::Slice<const uint8_t> data) : inner({data.data(), data.size()}) {}

std::unique_ptr<Sarc> sarc_from_binary(const rust::Slice<const uint8_t> data)
{
    return std::unique_ptr<Sarc>(new Sarc(data));
}

std::unique_ptr<std::vector<SarcFile>> Sarc::get_files() const {
    std::vector<SarcFile> files;
    for (const oead::Sarc::File file: inner.GetFiles()) {
        files.push_back(SarcFile(file));
    }
    return std::make_unique<std::vector<SarcFile>>(files);
}

rust::Slice<const uint8_t> Sarc::get_file_data(const rust::Str name) const {
    auto maybe_file = inner.GetFile(std::string_view(name.data(), name.size()));
    if (!maybe_file.has_value()) {
        throw std::runtime_error("File not found in SARC");
    }
    auto file = maybe_file.value();
    return rust::Slice<const uint8_t>(file.data.data(), file.data.size());
}

rust::Slice<const uint8_t> Sarc::idx_file_data(const uint16_t idx) const {
    auto file = inner.GetFile(idx);
    return rust::Slice(file.data.data(), file.data.size());
}

rust::Str Sarc::idx_file_name(const uint16_t idx) const {
    auto file = inner.GetFile(idx);
    return rust::Str(file.name.data(), file.name.size());
}

uint16_t Sarc::num_files() const {
    return inner.GetNumFiles();
}

uint32_t Sarc::get_offset() const {
    return inner.GetDataOffset();
}

size_t Sarc::guess_align() const {
    return inner.GuessMinAlignment();
}

bool Sarc::big_endian() const {
    return (inner.GetEndianness() == oead::util::Endianness::Big);
}

bool Sarc::files_eq(const Sarc& other) const {
    return this->inner.AreFilesEqual(other.inner);
}
