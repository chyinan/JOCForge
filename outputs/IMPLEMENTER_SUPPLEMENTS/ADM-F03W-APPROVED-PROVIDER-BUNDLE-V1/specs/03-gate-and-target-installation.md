# F03G Source gate与目标安装

本文件定义feature之后的gate及拟合之后的目标slot安装。selector、投影、fit、非负求解与marker另行定义；它们不能由本文件的安装算法代替。目标为全部必要非秘密规则完成隔离，当前剩余部分仍未就绪。

## F03G.1 Source gate

输入为band-major 24×48 finite F32 feature，以及按source0～47排序的两项整数flag：processing_inactive、selection_suppressed，取0/1。对应flag producer和跨callback feature state须由F03/F04完整定义。

每source的sum从F32正零开始，按band0～23逐项F32相加。selection_suppressed=0且sum>=R32(0.5)时进入candidate集合，按source递增排列；相等0.5入选。processing_inactive=0时进入outer solver调用序列，也按source递增排列。两个集合独立，不能把candidate等同于全部solver source。

候选feature矩阵按band递增、候选source递增重排，形状24×candidate_count。需要计算response column的source为两个集合的并集，递增且去重。所有48项sum、两个序列及重排feature作为明确输出；不读取参考candidate map。

没有candidate时selector不调用，返回空selected集合；仍执行当前callback的后续状态/quiet marker流程，不能直接丢弃整个callback。已有target state是否保持由F03G.3定义。

## F03G.2 输入state、距离与成本

目标state有15个slot：XYZ[15][3] finite F32、ID[15] U32、attribute[15] U32、previous XYZ/ID/attribute以及active[15]整数0/1。selected profile冷构造器由 `parameters/f03-selected-target-cold-v1.json` 精确提供：current/previous XYZ为45个positive-zero U32词，current/previous ID为15个`0xffffffff`，current/previous attribute均为15个`0x00000b00`，active与previous marker active为15个0。`0x00000b00`是F01B/F01GL prefix/bed的规范packed attribute，也与已验证U04 retained-target初始化一致；因此首次安装一个或多个B00 target时，multiplicity guard有15个旧slot可用。实现必须解析该provider，不再向布局或selector猜测cold值。F05R56 reader/metadata snapshot的attribute=0属于另一个owner的独立冷状态，不能反向覆盖本构造器。

selected输入有n项XYZ/ID/attribute，0<=n<=15，来自selector与projection；不是直接按输入track数取前15项。当前支持每种新attribute的出现次数不超过旧15-slot state中该attribute的次数。计数包含active=0的保留slot，因为这些slot正是首次安装的可分配容量；不设first-install豁免。selector输出的ID/attribute顺序经F03U保持一一对应，F03U只可能按bit0抑制位置写回，不改ID或attribute；安装输入必须携带同一代的这三组字段。超过时拒绝UNSUPPORTED_ATTRIBUTE_MULTIPLICITY，不能猜测未定义的特殊成本分支。

对旧slot r、新selected列c，distance从正零开始，按X/Y/Z顺序：delta=old_coordinate⊖new_coordinate，distance=distance⊕(delta⊗delta)。再cost=((distance⊕R32(20×int(old_ID!=new_ID)))⊕R32(100×int(old_attribute!=new_attribute)))⊕正零。

成本为15×n旧row-major/new-column矩阵。支持域要求所有cost有限且小于最大有限F32；超范围按I03失败，不用NaN/Infinity或FLT_MAX作有效成员。ID/attribute比较是整数精确相等，不用浮点距离代替。

## F03G.3 贪心映射与提交

n=0时mapping为15项-1，cost矩阵空；XYZ/ID/attributes、previous字段和active flags完整保持旧值。尤其不在这里把旧active全部清0；quiet marker可在后续阶段更新active。

n>0时mapping初始化-1，工作cost复制F03G.2。执行恰好n次选择：best初始化最大有限F32，按r=0～14、c=0～n-1扫描；仅当best>cost时替换best及所选pair。因此相等成本选择扫描中最早row/column。把mapping[r]=c，再将整row和整column的工作cost设为最大有限F32，防止再次选择。

无法选择有限pair时失败。该算法是全矩阵重复最小值的贪心，不是按旧row逐一匹配，也不是Hungarian最优分配；不能优化成另一种tie/assignment语义。

逐旧slot提交：mapping=-1时active=0，XYZ/ID/attribute及previous字段保持原值；mapping有效时active=1，先将该slot旧当前XYZ/ID/attribute复制到previous，再安装对应selected当前字段。未匹配slot不更新previous快照。输出mapping、原cost位模式和完整next state；工作cost遮蔽矩阵不是下一callback状态。

下一阶段destinations取安装后active=1的slot，按slot递增；它们的数量可能与本次空selected不同，不能用n=0强行覆盖保留的active。marker提交后再保存最终active/previous-marker状态。

## F03G.4 重置、失败及验收

reset由完整target constructor恢复所有当前及previous字段，不只清mapping。shape/type、attribute multiplicity、非有限成本等入口错误不产生next state；programme按I03终止。该保证不推广为上游selector或projection全部回滚。

验收包含sum恰好0.5、两个gate集合不同、n=0保留旧active、cold→单个B00、cold→prefix/bed B00组合、mixed attribute、全tie的最早row/column、相同ID复用、rectangular映射及full reset/replay。判别向量见`test-vectors/F03-COLD-ATTRIBUTE-INPUT-VECTORS.json`和`test-vectors/F03-COLD-ATTRIBUTE-VERIFICATION.json`。合成old/new state是显式输入，不作为生产constructor替代物。
