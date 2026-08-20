// =================================================================
// BinaryDecoder — Zero-Copy / Binary Packet Unpacker for Tauri IPC
// =================================================================

(function (root, factory) {
    if (typeof module === 'object' && module.exports) {
        module.exports = factory();
    } else {
        root.BinaryDecoder = factory();
    }
}(typeof self !== 'undefined' ? self : this, function () {
    const textDecoder = new TextDecoder('utf-8');

    function toUint8Array(data) {
        if (!data) return null;
        if (data instanceof Uint8Array) return data;
        if (data instanceof ArrayBuffer) return new Uint8Array(data);
        if (ArrayBuffer.isView(data)) return new Uint8Array(data.buffer, data.byteOffset, data.byteLength);
        if (Array.isArray(data)) return new Uint8Array(data);
        return null;
    }

    // Phase 2-3 Integration: Decode quantized frame format (65% bandwidth savings)
    function decodePtsFrameQuantized(dv, bytes) {
        const pointCount = dv.getUint32(8, true);
        const lineCount = dv.getUint32(12, true);
        const totalPoints = dv.getUint32(16, true);
        const fileNameLen = dv.getUint32(20, true);
        const bboxLabelLen = dv.getUint32(24, true);
        const shaderStatusLen = dv.getUint32(28, true);
        const overlay1Len = dv.getUint32(32, true);
        const overlay2Len = dv.getUint32(36, true);

        let offset = 40;

        // Dequantize points (7 bytes each: u16 x, u16 y, u8 r, u8 cr, u8 a)
        // Assume screen is 1920x1080 for denormalization (conservative estimate)
        const screenWidth = window.innerWidth || 1920;
        const screenHeight = window.innerHeight || 1080;

        const points = new Array(pointCount);
        const pointsBytes = bytes.subarray(offset, offset + pointCount * 7);

        for (let i = 0; i < pointCount; i++) {
            const idx = i * 7;
            // Dequantize: convert u16 (0-65535) back to float screen coords
            const x_u16 = pointsBytes[idx] | (pointsBytes[idx + 1] << 8);
            const y_u16 = pointsBytes[idx + 2] | (pointsBytes[idx + 3] << 8);
            const radius_u8 = pointsBytes[idx + 4];
            const coreRadius_u8 = pointsBytes[idx + 5];
            const alpha_u8 = pointsBytes[idx + 6];

            points[i] = {
                x: (x_u16 / 65535.0) * screenWidth,
                y: (y_u16 / 65535.0) * screenHeight,
                radius: radius_u8 / 255.0,
                core_radius: coreRadius_u8 / 255.0,
                alpha: alpha_u8 / 255.0
            };
        }
        offset += pointCount * 7;

        // Dequantize lines (8 bytes each: u16 x1, u16 y1, u16 x2, u16 y2)
        const boxLines = new Array(lineCount);
        const linesBytes = bytes.subarray(offset, offset + lineCount * 8);

        for (let i = 0; i < lineCount; i++) {
            const idx = i * 8;
            const x1_u16 = linesBytes[idx] | (linesBytes[idx + 1] << 8);
            const y1_u16 = linesBytes[idx + 2] | (linesBytes[idx + 3] << 8);
            const x2_u16 = linesBytes[idx + 4] | (linesBytes[idx + 5] << 8);
            const y2_u16 = linesBytes[idx + 6] | (linesBytes[idx + 7] << 8);

            boxLines[i] = {
                x1: (x1_u16 / 65535.0) * screenWidth,
                y1: (y1_u16 / 65535.0) * screenHeight,
                x2: (x2_u16 / 65535.0) * screenWidth,
                y2: (y2_u16 / 65535.0) * screenHeight
            };
        }
        offset += lineCount * 8;

        // Strings
        const strBytes = bytes.subarray(offset);
        let strOffset = 0;

        const fileName = textDecoder.decode(strBytes.subarray(strOffset, strOffset + fileNameLen));
        strOffset += fileNameLen;

        const bboxLabel = textDecoder.decode(strBytes.subarray(strOffset, strOffset + bboxLabelLen));
        strOffset += bboxLabelLen;

        const shaderStatus = textDecoder.decode(strBytes.subarray(strOffset, strOffset + shaderStatusLen));
        strOffset += shaderStatusLen;

        const overlay1 = textDecoder.decode(strBytes.subarray(strOffset, strOffset + overlay1Len));
        strOffset += overlay1Len;

        const overlay2 = textDecoder.decode(strBytes.subarray(strOffset, strOffset + overlay2Len));

        return {
            points: points,
            box_lines: boxLines,
            file_name: fileName,
            count: totalPoints,
            bbox_label: bboxLabel,
            shader_status: shaderStatus,
            overlay_text1: overlay1,
            overlay_text2: overlay2
        };
    }

    function decodePtsFrame(input) {
        if (!input) return null;
        const bytes = toUint8Array(input);
        if (!bytes || bytes.byteLength === 0) return null;
        if (bytes.byteLength < 40) {
            // Passthrough if already decoded JSON object
            return input;
        }

        const dv = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
        const magic = dv.getUint32(0, true);

        // Check if this is quantized format (Phase 2-3 Integration)
        if (magic === 0x4B505451) {
            return decodePtsFrameQuantized(dv, bytes);
        }

        // Otherwise, handle original non-quantized format
        if (magic !== 0x4B505453) {
            // Not a binary KPTS packet, return as-is
            return input;
        }

        const version = dv.getUint32(4, true);
        const pointCount = dv.getUint32(8, true);
        const lineCount = dv.getUint32(12, true);
        const totalPoints = dv.getUint32(16, true);
        const fileNameLen = dv.getUint32(20, true);
        const bboxLabelLen = dv.getUint32(24, true);
        const shaderStatusLen = dv.getUint32(28, true);
        const overlay1Len = dv.getUint32(32, true);
        const overlay2Len = dv.getUint32(36, true);

        let offset = 40;

        // Points
        const points = new Array(pointCount);
        for (let i = 0; i < pointCount; i++) {
            const x = dv.getFloat32(offset, true);
            const y = dv.getFloat32(offset + 4, true);
            const radius = dv.getFloat32(offset + 8, true);
            const coreRadius = dv.getFloat32(offset + 12, true);
            const alpha = dv.getFloat32(offset + 16, true);
            points[i] = {
                x: x,
                y: y,
                radius: radius,
                core_radius: coreRadius,
                alpha: alpha
            };
            offset += 20;
        }

        // Lines
        const boxLines = new Array(lineCount);
        for (let i = 0; i < lineCount; i++) {
            const x1 = dv.getFloat32(offset, true);
            const y1 = dv.getFloat32(offset + 4, true);
            const x2 = dv.getFloat32(offset + 8, true);
            const y2 = dv.getFloat32(offset + 12, true);
            boxLines[i] = {
                x1: x1,
                y1: y1,
                x2: x2,
                y2: y2
            };
            offset += 16;
        }

        // Strings
        const strBytes = bytes.subarray(offset);
        let strOffset = 0;

        const fileName = textDecoder.decode(strBytes.subarray(strOffset, strOffset + fileNameLen));
        strOffset += fileNameLen;

        const bboxLabel = textDecoder.decode(strBytes.subarray(strOffset, strOffset + bboxLabelLen));
        strOffset += bboxLabelLen;

        const shaderStatus = textDecoder.decode(strBytes.subarray(strOffset, strOffset + shaderStatusLen));
        strOffset += shaderStatusLen;

        const overlay1 = textDecoder.decode(strBytes.subarray(strOffset, strOffset + overlay1Len));
        strOffset += overlay1Len;

        const overlay2 = textDecoder.decode(strBytes.subarray(strOffset, strOffset + overlay2Len));

        return {
            points: points,
            box_lines: boxLines,
            file_name: fileName,
            count: totalPoints,
            bbox_label: bboxLabel,
            shader_status: shaderStatus,
            overlay_text1: overlay1,
            overlay_text2: overlay2
        };
    }

    function decodePtsPoints(input) {
        if (!input) return null;
        const bytes = toUint8Array(input);
        if (!bytes || bytes.byteLength < 40) {
            return input;
        }

        const dv = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
        const magic = dv.getUint32(0, true);
        if (magic !== 0x50545350) {
            return input;
        }

        const pointCount = dv.getUint32(8, true);
        const bboxMin = [
            dv.getFloat32(16, true),
            dv.getFloat32(20, true),
            dv.getFloat32(24, true)
        ];
        const bboxMax = [
            dv.getFloat32(28, true),
            dv.getFloat32(32, true),
            dv.getFloat32(36, true)
        ];

        let offset = 40;
        const points = new Array(pointCount);
        for (let i = 0; i < pointCount; i++) {
            points[i] = [
                dv.getFloat32(offset, true),
                dv.getFloat32(offset + 4, true),
                dv.getFloat32(offset + 8, true)
            ];
            offset += 12;
        }

        return {
            points: points,
            count: pointCount,
            bbox_min: bboxMin,
            bbox_max: bboxMax
        };
    }

    function decodeWaveObj(input) {
        if (!input) return null;
        const bytes = toUint8Array(input);
        if (!bytes || bytes.byteLength < 56) {
            return input;
        }

        const dv = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
        const magic = dv.getUint32(0, true);
        if (magic !== 0x4D455348) {
            return input;
        }

        const vertexCount = dv.getUint32(8, true);
        const faceCount = dv.getUint32(12, true);
        const triangleCount = dv.getUint32(16, true);
        const edgeCount = dv.getUint32(20, true);
        const normalCount = dv.getUint32(24, true);

        const bboxMin = [
            dv.getFloat32(32, true),
            dv.getFloat32(36, true),
            dv.getFloat32(40, true)
        ];
        const bboxMax = [
            dv.getFloat32(44, true),
            dv.getFloat32(48, true),
            dv.getFloat32(52, true)
        ];

        let offset = 56;

        // Points
        const points = new Array(vertexCount);
        for (let i = 0; i < vertexCount; i++) {
            points[i] = [
                dv.getFloat32(offset, true),
                dv.getFloat32(offset + 4, true),
                dv.getFloat32(offset + 8, true)
            ];
            offset += 12;
        }

        // Triangles
        const triangles = new Array(triangleCount);
        for (let i = 0; i < triangleCount; i++) {
            triangles[i] = [
                dv.getUint32(offset, true),
                dv.getUint32(offset + 4, true),
                dv.getUint32(offset + 8, true)
            ];
            offset += 12;
        }

        // Edges
        const edges = new Array(edgeCount);
        for (let i = 0; i < edgeCount; i++) {
            edges[i] = [
                dv.getUint32(offset, true),
                dv.getUint32(offset + 4, true)
            ];
            offset += 8;
        }

        // Normals
        const normals = new Array(normalCount);
        for (let i = 0; i < normalCount; i++) {
            normals[i] = [
                dv.getFloat32(offset, true),
                dv.getFloat32(offset + 4, true),
                dv.getFloat32(offset + 8, true)
            ];
            offset += 12;
        }

        return {
            points: points,
            triangles: triangles,
            edges: edges,
            normals: normals,
            vertex_count: vertexCount,
            face_count: faceCount,
            bbox_min: bboxMin,
            bbox_max: bboxMax
        };
    }

    return {
        decodePtsFrame: decodePtsFrame,
        decodePtsPoints: decodePtsPoints,
        decodeWaveObj: decodeWaveObj
    };
}));
