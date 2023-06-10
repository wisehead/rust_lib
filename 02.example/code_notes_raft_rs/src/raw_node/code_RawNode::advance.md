#1.RawNode::advance

```
RawNode::advance
--let applied = self.commit_since_index;
--let light_rd = self.advance_append(rd);
--self.advance_apply_to(applied);
```