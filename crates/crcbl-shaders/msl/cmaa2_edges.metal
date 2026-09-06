#include <metal_stdlib>
#include <metal_math>
#include <metal_texture>
using namespace metal;

#line 70 "shaders/cmaa2_edges.slang"
struct Cmaa2Params_0
{
    uint viewport_x_0;
    uint viewport_y_0;
    uint candidate_capacity_0;
    uint item_capacity_0;
};


#line 284
struct KernelContext_0
{
    atomic<uint> device* control_0;
    Cmaa2Params_0 constant* params_0;
    atomic<uint> device* accum_0;
    texture2d<float, access::sample> source_0;
    uint device* edges_0;
    uint device* candidates_0;
};


#line 218
[[kernel]] void clearMain(uint3 thread_0 [[thread_position_in_grid]], atomic<uint> device* control_1 [[buffer(1)]], Cmaa2Params_0 constant* params_1 [[buffer(0)]], atomic<uint> device* accum_1 [[buffer(4)]], texture2d<float, access::sample> source_1 [[texture(0)]], uint device* edges_1 [[buffer(2)]], uint device* candidates_1 [[buffer(3)]])
{

#line 218
    thread KernelContext_0 kernelContext_0;

#line 218
    (&kernelContext_0)->control_0 = control_1;

#line 218
    (&kernelContext_0)->params_0 = params_1;

#line 218
    (&kernelContext_0)->accum_0 = accum_1;

#line 218
    (&kernelContext_0)->source_0 = source_1;

#line 218
    (&kernelContext_0)->edges_0 = edges_1;

#line 218
    (&kernelContext_0)->candidates_0 = candidates_1;

    uint _S1 = thread_0.x;

#line 220
    if(_S1 >= 2U)
    {
        return;
    }
    atomic_store_explicit((&kernelContext_0)->control_0+_S1, 0U, memory_order_relaxed);
    return;
}


#line 195
float luma_of_0(float3 color_0)
{
    return sqrt(dot(color_0, float3(0.2125999927520752f, 0.71520000696182251f, 0.07220000028610229f)));
}


#line 205
float luma_at_0(int2 texel_0, KernelContext_0 thread* kernelContext_1)
{

    int3 _S2 = int3(clamp(texel_0, int2(int(0), int(0)), int2(int(kernelContext_1->params_0->viewport_x_0) - int(1), int(kernelContext_1->params_0->viewport_y_0) - int(1))), int(0));

#line 208
    return luma_of_0(((kernelContext_1->source_0).read(vec<uint,2>(((_S2)).xy), uint(((_S2)).z))).xyz);
}


#line 240
[[kernel]] void edgesMain(uint3 thread_1 [[thread_position_in_grid]], atomic<uint> device* control_2 [[buffer(1)]], Cmaa2Params_0 constant* params_2 [[buffer(0)]], atomic<uint> device* accum_2 [[buffer(4)]], texture2d<float, access::sample> source_2 [[texture(0)]], uint device* edges_2 [[buffer(2)]], uint device* candidates_2 [[buffer(3)]])
{

#line 240
    thread KernelContext_0 kernelContext_2;

#line 240
    (&kernelContext_2)->control_0 = control_2;

#line 240
    (&kernelContext_2)->params_0 = params_2;

#line 240
    (&kernelContext_2)->accum_0 = accum_2;

#line 240
    (&kernelContext_2)->source_0 = source_2;

#line 240
    (&kernelContext_2)->edges_0 = edges_2;

#line 240
    (&kernelContext_2)->candidates_0 = candidates_2;

    uint index_0 = thread_1.x;

    if(index_0 >= (params_2->viewport_x_0 * params_2->viewport_y_0))
    {
        return;
    }
    uint x_0 = index_0 % params_2->viewport_x_0;
    uint y_0 = index_0 / params_2->viewport_x_0;
    int2 texel_1 = int2(int(x_0), int(y_0));

#line 250
    uint word_0 = 0U;



    for(;;)
    {

#line 254
        if(word_0 < 4U)
        {
        }
        else
        {

#line 254
            break;
        }
        atomic_store_explicit((&kernelContext_2)->accum_0+(index_0 * 4U + word_0), 0U, memory_order_relaxed);

#line 254
        word_0 = word_0 + 1U;

#line 254
    }

#line 254
    float _S3 = luma_at_0(texel_1, &kernelContext_2);

#line 254
    float _S4 = luma_at_0(texel_1 + int2(int(-1), int(0)), &kernelContext_2);

#line 254
    float _S5 = luma_at_0(texel_1 + int2(int(0), int(-1)), &kernelContext_2);

#line 254
    float _S6 = luma_at_0(texel_1 + int2(int(1), int(0)), &kernelContext_2);

#line 254
    float _S7 = luma_at_0(texel_1 + int2(int(0), int(1)), &kernelContext_2);

#line 254
    float2 _S8 = float2(_S3) ;

#line 266
    float2 delta_0 = abs(_S8 - float2(_S4, _S5));

#line 271
    float2 other_0 = abs(_S8 - float2(_S6, _S7));
    float _S9 = max(max(delta_0.x, delta_0.y), max(other_0.x, other_0.y));
    float2 marked_0 = step(float2(0.10000000149011612f, 0.10000000149011612f), delta_0) * step(float2(_S9, _S9), float2(2.0f)  * delta_0);

#line 273
    bool _S10;


    if(x_0 > 0U)
    {

#line 276
        _S10 = (marked_0.x) > 0.0f;

#line 276
    }
    else
    {

#line 276
        _S10 = false;

#line 276
    }

#line 276
    uint bits_0;

#line 276
    if(_S10)
    {

#line 276
        bits_0 = 1U;

#line 276
    }
    else
    {

#line 276
        bits_0 = 0U;

#line 276
    }



    if(y_0 > 0U)
    {

#line 280
        _S10 = (marked_0.y) > 0.0f;

#line 280
    }
    else
    {

#line 280
        _S10 = false;

#line 280
    }

#line 280
    if(_S10)
    {

#line 280
        bits_0 = bits_0 | 2U;

#line 280
    }



    *((&kernelContext_2)->edges_0+index_0) = bits_0;

    if(bits_0 != 0U)
    {

#line 293
        uint slot_0 = atomic_fetch_add_explicit((&kernelContext_2)->control_0+int(0), 1U, memory_order_relaxed);
        if(slot_0 < ((&kernelContext_2)->params_0->candidate_capacity_0))
        {
            *((&kernelContext_2)->candidates_0+slot_0) = index_0;

#line 294
        }

#line 286
    }

#line 299
    return;
}

