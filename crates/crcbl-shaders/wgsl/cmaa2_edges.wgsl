@binding(2) @group(0) var<storage, read_write> control_0 : array<atomic<u32>>;

struct Cmaa2Params_std140_0
{
    @align(16) viewport_x_0 : u32,
    @align(4) viewport_y_0 : u32,
    @align(8) candidate_capacity_0 : u32,
    @align(4) item_capacity_0 : u32,
};

@binding(0) @group(0) var<uniform> params_0 : Cmaa2Params_std140_0;
@binding(5) @group(0) var<storage, read_write> accum_0 : array<atomic<u32>>;

@binding(1) @group(0) var source_0 : texture_2d<f32>;

@binding(3) @group(0) var<storage, read_write> edges_0 : array<u32>;

@binding(4) @group(0) var<storage, read_write> candidates_0 : array<u32>;

@compute
@workgroup_size(64, 1, 1)
fn clearMain(@builtin(global_invocation_id) thread_0 : vec3<u32>)
{
    var _S1 : u32 = thread_0.x;
    if(_S1 >= u32(2))
    {
        return;
    }
    atomicStore(&(control_0[_S1]), u32(0));
    return;
}

fn luma_of_0( color_0 : vec3<f32>) -> f32
{
    return sqrt(dot(color_0, vec3<f32>(0.2125999927520752f, 0.71520000696182251f, 0.07220000028610229f)));
}

fn luma_at_0( texel_0 : vec2<i32>) -> f32
{
    var _S2 : vec3<i32> = vec3<i32>(clamp(texel_0, vec2<i32>(i32(0), i32(0)), vec2<i32>(i32(params_0.viewport_x_0) - i32(1), i32(params_0.viewport_y_0) - i32(1))), i32(0));
    return luma_of_0((textureLoad((source_0), ((_S2)).xy, ((_S2)).z)).xyz);
}

@compute
@workgroup_size(64, 1, 1)
fn edgesMain(@builtin(global_invocation_id) thread_1 : vec3<u32>)
{
    var index_0 : u32 = thread_1.x;
    if(index_0 >= (params_0.viewport_x_0 * params_0.viewport_y_0))
    {
        return;
    }
    var x_0 : u32 = index_0 % params_0.viewport_x_0;
    var y_0 : u32 = index_0 / params_0.viewport_x_0;
    var texel_1 : vec2<i32> = vec2<i32>(i32(x_0), i32(y_0));
    var word_0 : u32 = u32(0);
    for(;;)
    {
        if(word_0 < u32(4))
        {
        }
        else
        {
            break;
        }
        atomicStore(&(accum_0[index_0 * u32(4) + word_0]), u32(0));
        word_0 = word_0 + u32(1);
    }
    var _S3 : vec2<f32> = vec2<f32>(luma_at_0(texel_1));
    var delta_0 : vec2<f32> = abs(_S3 - vec2<f32>(luma_at_0(texel_1 + vec2<i32>(i32(-1), i32(0))), luma_at_0(texel_1 + vec2<i32>(i32(0), i32(-1)))));
    var other_0 : vec2<f32> = abs(_S3 - vec2<f32>(luma_at_0(texel_1 + vec2<i32>(i32(1), i32(0))), luma_at_0(texel_1 + vec2<i32>(i32(0), i32(1)))));
    var _S4 : f32 = max(max(delta_0.x, delta_0.y), max(other_0.x, other_0.y));
    var marked_0 : vec2<f32> = step(vec2<f32>(0.10000000149011612f, 0.10000000149011612f), delta_0) * step(vec2<f32>(_S4, _S4), vec2<f32>(2.0f) * delta_0);
    var _S5 : bool;
    if(x_0 > u32(0))
    {
        _S5 = (marked_0.x) > 0.0f;
    }
    else
    {
        _S5 = false;
    }
    var bits_0 : u32;
    if(_S5)
    {
        bits_0 = u32(1);
    }
    else
    {
        bits_0 = u32(0);
    }
    if(y_0 > u32(0))
    {
        _S5 = (marked_0.y) > 0.0f;
    }
    else
    {
        _S5 = false;
    }
    if(_S5)
    {
        bits_0 = (bits_0 | (u32(2)));
    }
    edges_0[index_0] = bits_0;
    if(bits_0 != u32(0))
    {
        var slot_0 : u32 = atomicAdd(&(control_0[i32(0)]), u32(1));
        if(slot_0 < (params_0.candidate_capacity_0))
        {
            candidates_0[slot_0] = index_0;
        }
    }
    return;
}

