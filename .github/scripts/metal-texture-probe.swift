import Darwin
import Foundation
import Metal

guard let device = MTLCreateSystemDefaultDevice(),
      let queue = device.makeCommandQueue()
else {
    fputs("raw Metal probe: no device or queue\n", stderr)
    exit(2)
}

let sourceDescriptor = MTLTextureDescriptor.texture2DDescriptor(
    pixelFormat: .r8Unorm,
    width: 1,
    height: 1,
    mipmapped: false
)
sourceDescriptor.storageMode = .shared
sourceDescriptor.usage = [.shaderRead]
guard let sourceTexture = device.makeTexture(descriptor: sourceDescriptor) else {
    fputs("raw Metal probe: source texture creation failed\n", stderr)
    exit(3)
}
sourceTexture.label = "raw shader-read source"
var white: UInt8 = 255
sourceTexture.replace(
    region: MTLRegionMake2D(0, 0, 1, 1),
    mipmapLevel: 0,
    withBytes: &white,
    bytesPerRow: 1
)

let targetDescriptor = MTLTextureDescriptor.texture2DDescriptor(
    pixelFormat: .rgba8Unorm,
    width: 8,
    height: 8,
    mipmapped: false
)
targetDescriptor.storageMode = .shared
targetDescriptor.usage = [.renderTarget]
guard let targetTexture = device.makeTexture(descriptor: targetDescriptor) else {
    fputs("raw Metal probe: target texture creation failed\n", stderr)
    exit(4)
}
targetTexture.label = "raw render target"

let shader = """
#include <metal_stdlib>
using namespace metal;

struct VertexOut { float4 position [[position]]; };

vertex VertexOut vertexMain(uint id [[vertex_id]]) {
    const float2 points[3] = { float2(-1.0, -1.0), float2(3.0, -1.0), float2(-1.0, 3.0) };
    VertexOut out;
    out.position = float4(points[id], 0.0, 1.0);
    return out;
}

fragment float4 fragmentMain(
    texture2d<float, access::sample> image [[texture(0)]],
    sampler imageSampler [[sampler(0)]])
{
    return float4(image.sample(imageSampler, float2(0.5)).r, 0.0, 0.0, 1.0);
}
"""

let library: MTLLibrary
do {
    library = try device.makeLibrary(source: shader, options: nil)
} catch {
    fputs("raw Metal probe: shader compile failed: \(error)\n", stderr)
    exit(5)
}

let pipelineDescriptor = MTLRenderPipelineDescriptor()
pipelineDescriptor.label = "raw sampled-texture pipeline"
pipelineDescriptor.vertexFunction = library.makeFunction(name: "vertexMain")
pipelineDescriptor.fragmentFunction = library.makeFunction(name: "fragmentMain")
pipelineDescriptor.colorAttachments[0].pixelFormat = .rgba8Unorm
let pipeline: MTLRenderPipelineState
do {
    pipeline = try device.makeRenderPipelineState(descriptor: pipelineDescriptor)
} catch {
    fputs("raw Metal probe: pipeline creation failed: \(error)\n", stderr)
    exit(6)
}

let samplerDescriptor = MTLSamplerDescriptor()
samplerDescriptor.minFilter = .nearest
samplerDescriptor.magFilter = .nearest
guard let sampler = device.makeSamplerState(descriptor: samplerDescriptor),
      let commands = queue.makeCommandBuffer()
else {
    fputs("raw Metal probe: sampler or command buffer creation failed\n", stderr)
    exit(7)
}

let pass = MTLRenderPassDescriptor()
pass.colorAttachments[0].texture = targetTexture
pass.colorAttachments[0].loadAction = .clear
pass.colorAttachments[0].storeAction = .store
pass.colorAttachments[0].clearColor = MTLClearColorMake(0.0, 0.0, 0.0, 1.0)
guard let encoder = commands.makeRenderCommandEncoder(descriptor: pass) else {
    fputs("raw Metal probe: render encoder creation failed\n", stderr)
    exit(8)
}
encoder.label = "raw sampled-texture draw"
encoder.setRenderPipelineState(pipeline)
encoder.setFragmentTexture(sourceTexture, index: 0)
encoder.setFragmentSamplerState(sampler, index: 0)
encoder.drawPrimitives(type: .triangle, vertexStart: 0, vertexCount: 3)
encoder.endEncoding()
commands.commit()
commands.waitUntilCompleted()

guard commands.status == .completed else {
    fputs("raw Metal probe: command buffer status \(commands.status.rawValue), error \(String(describing: commands.error))\n", stderr)
    exit(9)
}

var pixels = [UInt8](repeating: 0, count: 8 * 8 * 4)
targetTexture.getBytes(
    &pixels,
    bytesPerRow: 8 * 4,
    from: MTLRegionMake2D(0, 0, 8, 8),
    mipmapLevel: 0
)
let centre = (4 * 8 + 4) * 4
let texel = Array(pixels[centre ..< centre + 4])
print("raw Metal probe: device=\(device.name) sourceUsage=\(sourceTexture.usage.rawValue) centre=\(texel)")
guard texel == [255, 0, 0, 255] else {
    fputs("raw Metal probe: sampled texture did not reach the render target\n", stderr)
    exit(10)
}
