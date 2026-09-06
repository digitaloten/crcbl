struct Cmaa2Params_std140_0
{
    @align(16) viewport_x_0 : u32,
    @align(4) viewport_y_0 : u32,
};

@binding(2) @group(0) var<uniform> params_0 : Cmaa2Params_std140_0;
@binding(0) @group(0) var source_0 : texture_2d<f32>;

@binding(1) @group(0) var<storage, read> accum_0 : array<u32>;

struct FullscreenOutput_0
{
    @builtin(position) position_0 : vec4<f32>,
    @location(0) uv_0 : vec2<f32>,
};

@vertex
fn vertexMain(@builtin(vertex_index) index_0 : u32) -> FullscreenOutput_0
{
    var output_0 : FullscreenOutput_0;
    var _S1 : vec2<f32> = vec2<f32>(f32((((index_0 << (u32(1)))) & (u32(2)))), f32((index_0 & (u32(2)))));
    output_0.uv_0 = _S1;
    output_0.position_0 = vec4<f32>(_S1 * vec2<f32>(2.0f, -2.0f) + vec2<f32>(-1.0f, 1.0f), 0.0f, 1.0f);
    return output_0;
}

struct pixelOutput_0
{
    @location(0) output_1 : vec4<f32>,
};

struct pixelInput_0
{
    @location(0) uv_1 : vec2<f32>,
};

@fragment
fn fragmentMain( _S2 : pixelInput_0, @builtin(position) position_1 : vec4<f32>) -> pixelOutput_0
{
    var texel_0 : vec2<u32> = min(vec2<u32>(position_1.xy), vec2<u32>(params_0.viewport_x_0 - u32(1), params_0.viewport_y_0 - u32(1)));
    var _S3 : vec3<i32> = vec3<i32>(vec2<i32>(texel_0), i32(0));
    var own_0 : vec3<f32> = (textureLoad((source_0), ((_S3)).xy, ((_S3)).z)).xyz;
    var _S4 : u32 = (texel_0.y * params_0.viewport_x_0 + texel_0.x) * u32(4);
    var weight_0 : f32 = f32(accum_0[_S4 + u32(3)]) * 9.5367431640625e-07f;
    if(weight_0 <= 0.0f)
    {
        var _S5 : pixelOutput_0 = pixelOutput_0( vec4<f32>(own_0, 1.0f) );
        return _S5;
    }
    var blended_0 : vec3<f32> = vec3<f32>(f32(accum_0[_S4]), f32(accum_0[_S4 + u32(1)]), f32(accum_0[_S4 + u32(2)])) * vec3<f32>(9.5367431640625e-07f);
    if(weight_0 >= 1.0f)
    {
        var _S6 : pixelOutput_0 = pixelOutput_0( vec4<f32>(blended_0 / vec3<f32>(weight_0), 1.0f) );
        return _S6;
    }
    var _S7 : pixelOutput_0 = pixelOutput_0( vec4<f32>(own_0 * vec3<f32>((1.0f - weight_0)) + blended_0, 1.0f) );
    return _S7;
}

