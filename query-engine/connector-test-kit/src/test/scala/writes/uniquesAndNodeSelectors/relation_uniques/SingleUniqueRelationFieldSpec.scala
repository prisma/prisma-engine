package writes.uniquesAndNodeSelectors.relation_uniques

import org.scalatest.{FlatSpec, Matchers}
import util._

// 1) Checks if relation fields in @unique in any constellation work with our mutations.
// Possible relation cardinalities:
// - 1!:1!
// - 1!:1
// - 1!:M
//
// 2) Checks basic cursor functionality.
class SingleUniqueRelationFieldSpec extends FlatSpec with Matchers with ApiSpecBase {
  // Mutations in this test:
  //  create         | root   | checked
  //  update         | root   | checked
  //  delete         | root   | not possible (1!:1!)
  //  upsert         | root   | checked
  //  updateMany     | root   | unnecessary
  //  deleteMany     | root   | unnecessary
  //  nested create  | create | checked
  //  nested update  | update | checked
  //  nested connect | -      | not possible (1!:1!)
  //  nested disconn | -      | not possible (1!:1!)
  //  nested delete  | -      | not possible (1!:1!)
  //  nested set     | -      | not possible (1!:1!)
  //  nested upsert  | -      | not possible (1!:1!)
  //  nested deleteM | -      | not possible (1!:1!)
  //  nested updateM | -      | not possible (1!:1!)
  "Using a unique that is also a 1!:1! single-field relation" should "work" in {
    val project = ProjectDsl.fromString {
      s"""
         |model Parent {
         |  id    Int    @id
         |  child Child  @relation(references: [id]) @unique
         |  p     String
         |}
         |
         |model Child {
         |  id     Int    @id
         |  c      String
         |  parent Parent
         |}
       """
    }
    database.setup(project)

    val res1 = server.query(
      """
        |mutation {
        |  createParent(
        |    data: {
        |      id: 1
        |      p: "Parent"
        |      child: {
        |        create: {
        |          id: 1,
        |          c: "Child"
        |        }
        |      }
        |    }
        |  ) {
        |    id
        |    p
        |    child{
        |       id
        |       c
        |    }
        |  }
        |}
      """,
      project
    )

    res1.toString() should be("{\"data\":{\"createParent\":{\"id\":1,\"p\":\"Parent\",\"child\":{\"id\":1,\"c\":\"Child\"}}}}")

    val res2 = server.query(
      """
        |mutation {
        |  updateParent(where: { child: 1 } data: { p: "UpdatedParent" }) {
        |    p
        |  }
        |}
      """,
      project
    )

    res2.toString() should be("{\"data\":{\"updateParent\":{\"p\":\"UpdatedParent\"}}}")

    val res3 = server.query(
      """
        |mutation {
        |  updateChild(where: { id: 1 } data: { parent: { update: { p: "UpdatedFromChild" }}}) {
        |    parent { p }
        |  }
        |}
      """,
      project
    )

    res3.toString() should be("{\"data\":{\"updateChild\":{\"parent\":{\"p\":\"UpdatedFromChild\"}}}}")

    val res4 = server.query(
      """
        |mutation {
        |  upsertParent(
        |    where: { child: 2 }
        |    update: { p: "Doesn't matter" }
        |    create: { id: 2, p: "Parent2", child: { create: { id: 2, c: "Child2" } } }
        |  ) {
        |    id
        |    child {
        |      id
        |    }
        |  }
        |}
        |
      """,
      project
    )

    res4.toString() should be("{\"data\":{\"upsertParent\":{\"id\":2,\"child\":{\"id\":2}}}}")
  }

  // Mutations in this test:
  //  create         | root   | checked
  //  update         | root   | checked
  //  delete         | root   | not possible (1!:1!)
  //  upsert         | root   | checked
  //  updateMany     | root   | unnecessary
  //  deleteMany     | root   | unnecessary
  //  nested create  | create | checked
  //  nested update  | update | checked
  //  nested connect | -      | not possible (1!:1!)
  //  nested disconn | -      | not possible (1!:1!)
  //  nested delete  | -      | not possible (1!:1!)
  //  nested set     | -      | not possible (1!:1!)
  //  nested upsert  | -      | not possible (1!:1!)
  //  nested deleteM | -      | not possible (1!:1!)
  //  nested updateM | -      | not possible (1!:1!)
  "Using a unique that is also a 1!:1! multi-field relation" should "work" in {
    val project = ProjectDsl.fromString {
      s"""
         |model Parent {
         |  id    Int    @id
         |  child Child  @relation(references: [id, c]) @unique
         |  p     String
         |}
         |
         |model Child {
         |  id     Int    @id
         |  c      String
         |  parent Parent
         |
         |  @@unique([id, c])
         |}
       """
    }
    database.setup(project)

    val res1 = server.query(
      """
        |mutation {
        |  createParent(
        |    data: {
        |      id: 1
        |      p: "Parent"
        |      child: {
        |        create: {
        |          id: 1,
        |          c: "Child"
        |        }
        |      }
        |    }
        |  ) {
        |    id
        |    p
        |    child{
        |       id
        |       c
        |    }
        |  }
        |}
      """,
      project
    )

    res1.toString() should be("{\"data\":{\"createParent\":{\"id\":1,\"p\":\"Parent\",\"child\":{\"id\":1,\"c\":\"Child\"}}}}")

    val res2 = server.query(
      """
        |mutation {
        |  updateParent(where: { child: { child_id: 1, child_c: "Child" } } data: { p: "UpdatedParent" }) {
        |    p
        |  }
        |}
      """,
      project
    )

    res2.toString() should be("{\"data\":{\"updateParent\":{\"p\":\"UpdatedParent\"}}}")

    val res3 = server.query(
      """
        |mutation {
        |  updateChild(where: { id: 1 } data: { parent: { update: { p: "UpdatedFromChild" }}}) {
        |    parent { p }
        |  }
        |}
      """,
      project
    )

    res3.toString() should be("{\"data\":{\"updateChild\":{\"parent\":{\"p\":\"UpdatedFromChild\"}}}}")

    val res4 = server.query(
      """
        |mutation {
        |  upsertParent(
        |    where:  { child: { child_id: 2, child_c: "Child2" } }
        |    update: { p: "Doesn't matter" }
        |    create: { id: 2, p: "Parent2", child: { create: { id: 2, c: "Child2" } } }
        |  ) {
        |    id
        |    child {
        |      id
        |    }
        |  }
        |}
        |
      """,
      project
    )

    res4.toString() should be("{\"data\":{\"upsertParent\":{\"id\":2,\"child\":{\"id\":2}}}}")
  }

  // Mutations in this test:
  //  create         | root   | checked
  //  update         | root   | checked
  //  delete         | root   | checked
  //  upsert         | root   | checked
  //  updateMany     | root   | unnecessary
  //  deleteMany     | root   | unnecessary
  //  nested create  | create | checked
  //  nested update  | update | checked
  //  nested connect | create | checked
  //  nested connect | update | checked
  //  nested delete  | -      | checked
  //  nested upsert  | update | checked
  //  nested disconn | -      | not possible (1!:1)
  //  nested set     | -      | not possible (1!:1)
  //  nested deleteM | -      | not possible (1!:1)
  //  nested updateM | -      | not possible (1!:1)
  "Using a unique that is also a 1!:1 single-field relation" should "work" in {
    val project = ProjectDsl.fromString {
      s"""
         |model Parent {
         |  id    Int    @id
         |  child Child  @relation(references: [id]) @unique
         |  p     String
         |}
         |
         |model Child {
         |  id     Int     @id
         |  c      String
         |  parent Parent?
         |
         |  @@unique([id, c])
         |}
       """
    }
    database.setup(project)

    val res1 = server.query(
      """
        |mutation {
        |  createParent(data: { id: 1, p: "Parent1", child: { create: { id: 1, c: "Child1" }}}) {
        |    p
        |    child {
        |       c
        |    }
        |  }
        |}
      """,
      project
    )

    res1.toString() should be("{\"data\":{\"createParent\":{\"p\":\"Parent1\",\"child\":{\"c\":\"Child1\"}}}}")

    val res2 = server.query(
      """
        |mutation {
        |  updateParent(where: { child: 1 } data: { p: "UpdatedParent1" }) {
        |    p
        |  }
        |}
      """,
      project
    )

    res2.toString() should be("{\"data\":{\"updateParent\":{\"p\":\"UpdatedParent1\"}}}")

    val res3 = server.query(
      """
        |mutation {
        |  updateChild(where: { id: 1 } data: { parent: { update: { p: "UpdateParent1FromChild" }}}) {
        |    parent { p }
        |  }
        |}
      """,
      project
    )

    res3.toString() should be("{\"data\":{\"updateChild\":{\"parent\":{\"p\":\"UpdateParent1FromChild\"}}}}")

    val res4 = server.query(
      """
        |mutation {
        |  upsertParent(
        |    where: { child: 2 }
        |    update: { p: "doesn't matter" }
        |    create: { id: 2, p: "Parent2", child: { create: { id: 2, c: "Child2" } } }
        |  ) {
        |    p
        |  }
        |}
        |
      """,
      project
    )

    res4.toString() should be("{\"data\":{\"upsertParent\":{\"p\":\"Parent2\"}}}")

    val res5 = server.query(
      """
        |mutation {
        |  deleteParent(
        |    where: {
        |      child: 2
        |    }
        |  ) {
        |    p
        |  }
        |}
      """,
      project
    )

    res5.toString() should be("{\"data\":{\"deleteParent\":{\"p\":\"Parent2\"}}}")

    val res6 = server.query(
      """
        |mutation {
        |  createParent(
        |    data: {
        |      id: 2
        |      p: "Parent2New",
        |      child: {
        |        connect: {
        |          id: 2
        |        }
        |      }
        |    }
        |  ) {
        |    p
        |    child {
        |      id
        |    }
        |  }
        |}
        |
      """,
      project
    )

    res6.toString() should be("{\"data\":{\"createParent\":{\"p\":\"Parent2New\",\"child\":{\"id\":2}}}}")

    val res7 = server.query(
      """
        |mutation {
        |  createChild(
        |    data: {
        |      id: 3
        |      c: "Child3",
        |    }
        |  ) {
        |    id
        |  }
        |}
        |
      """,
      project
    )

    res7.toString() should be("{\"data\":{\"createChild\":{\"id\":3}}}")

    val res8 = server.query(
      """
        |mutation {
        |  updateParent(
        |    where: {
        |      child: 2
        |    }
        |    data: {
        |      child: {
        |        connect: {
        |          id: 3
        |        }
        |      }
        |    }
        |  ) {
        |    child {
        |      id
        |    }
        |  }
        |}
      """,
      project
    )

    res8.toString() should be("{\"data\":{\"updateParent\":{\"child\":{\"id\":3}}}}")

    val res9 = server.query(
      """
        |mutation {
        |  updateChild(
        |    where: { id: 3 }
        |    data: {
        |      parent: {
        |        upsert: {
        |          create: {
        |            id: 3
        |            p: "Parent3",
        |          }
        |          update: {
        |            p: "doesn't matter"
        |          }
        |        }
        |      }
        |    }
        |  ) {
        |    id
        |    parent {
        |      child {
        |        id
        |      }
        |    }
        |  }
        |}
      """,
      project
    )

    res9.toString() should be("{\"data\":{\"updateChild\":{\"id\":3,\"parent\":{\"child\":{\"id\":3}}}}}")

    val res10 = server.query(
      """
        |mutation {
        |  updateChild(
        |    where: { id: 3 }
        |    data: {
        |      parent: {
        |        delete: true
        |      }
        |    }
        |  ) {
        |    id
        |    parent {
        |      child {
        |        id
        |      }
        |    }
        |  }
        |}
      """,
      project
    )

    res10.toString() should be("{\"data\":{\"updateChild\":{\"id\":3,\"parent\":null}}}")
  }

  // Mutations in this test:
  //  create         | root   | checked
  //  update         | root   | checked
  //  delete         | root   | checked
  //  upsert         | root   | checked
  //  updateMany     | root   | unnecessary
  //  deleteMany     | root   | unnecessary
  //  nested create  | create | checked
  //  nested update  | update | checked
  //  nested connect | create | checked
  //  nested connect | update | checked
  //  nested delete  | -      | checked
  //  nested upsert  | update | checked
  //  nested disconn | -      | not possible (1!:1)
  //  nested set     | -      | not possible (1!:1)
  //  nested deleteM | -      | not possible (1!:1)
  //  nested updateM | -      | not possible (1!:1)
  "Using a unique that is also a 1!:1 multi-field relation" should "work" in {
    val project = ProjectDsl.fromString {
      s"""
         |model Parent {
         |  id    Int    @id
         |  child Child  @relation(references: [id, c]) @unique
         |  p     String
         |}
         |
         |model Child {
         |  id     Int     @id
         |  c      String
         |  parent Parent?
         |
         |  @@unique([id, c])
         |}
       """
    }
    database.setup(project)

    val res1 = server.query(
      """
        |mutation {
        |  createParent(data: { id: 1, p: "Parent1", child: { create: { id: 1, c: "Child1" }}}) {
        |    p
        |    child {
        |       c
        |    }
        |  }
        |}
      """,
      project
    )

    res1.toString() should be("{\"data\":{\"createParent\":{\"p\":\"Parent1\",\"child\":{\"c\":\"Child1\"}}}}")

    val res2 = server.query(
      """
        |mutation {
        |  updateParent(where: { child: { child_id: 1, child_c: "Child1" } } data: { p: "UpdatedParent1" }) {
        |    p
        |  }
        |}
      """,
      project
    )

    res2.toString() should be("{\"data\":{\"updateParent\":{\"p\":\"UpdatedParent1\"}}}")

    val res3 = server.query(
      """
        |mutation {
        |  updateChild(where: { id: 1 } data: { parent: { update: { p: "UpdateParent1FromChild" }}}) {
        |    parent { p }
        |  }
        |}
      """,
      project
    )

    res3.toString() should be("{\"data\":{\"updateChild\":{\"parent\":{\"p\":\"UpdateParent1FromChild\"}}}}")

    val res4 = server.query(
      """
        |mutation {
        |  upsertParent(
        |    where: { child: { child_id: 2, child_c: "Child2" } }
        |    update: { p: "doesn't matter" }
        |    create: { id: 2, p: "Parent2", child: { create: { id: 2, c: "Child2" } } }
        |  ) {
        |    p
        |  }
        |}
        |
      """,
      project
    )

    res4.toString() should be("{\"data\":{\"upsertParent\":{\"p\":\"Parent2\"}}}")

    val res5 = server.query(
      """
        |mutation {
        |  deleteParent(
        |    where: {
        |      child: {
        |        child_id: 2,
        |        child_c: "Child2"
        |      }
        |    }
        |  ) {
        |    p
        |  }
        |}
      """,
      project
    )

    res5.toString() should be("{\"data\":{\"deleteParent\":{\"p\":\"Parent2\"}}}")

    val res6 = server.query(
      """
        |mutation {
        |  createParent(
        |    data: {
        |      id: 2
        |      p: "Parent2New",
        |      child: {
        |        connect: {
        |          id: 2
        |        }
        |      }
        |    }
        |  ) {
        |    p
        |    child {
        |      id
        |    }
        |  }
        |}
        |
      """,
      project
    )

    res6.toString() should be("{\"data\":{\"createParent\":{\"p\":\"Parent2New\",\"child\":{\"id\":2}}}}")

    val res7 = server.query(
      """
        |mutation {
        |  createChild(
        |    data: {
        |      id: 3
        |      c: "Child3",
        |    }
        |  ) {
        |    id
        |  }
        |}
        |
      """,
      project
    )

    res7.toString() should be("{\"data\":{\"createChild\":{\"id\":3}}}")

    val res8 = server.query(
      """
        |mutation {
        |  updateParent(
        |    where: {
        |      child: {
        |        child_id: 2
        |        child_c: "Child2"
        |      }
        |    }
        |    data: {
        |      child: {
        |        connect: {
        |          id: 3
        |        }
        |      }
        |    }
        |  ) {
        |    child {
        |      id
        |    }
        |  }
        |}
      """,
      project
    )

    res8.toString() should be("{\"data\":{\"updateParent\":{\"child\":{\"id\":3}}}}")

    val res9 = server.query(
      """
        |mutation {
        |  createChild(
        |    data: {
        |      id: 4
        |      c: "Child4",
        |    }
        |  ) {
        |    id
        |  }
        |}
        |
      """,
      project
    )

    res9.toString() should be("{\"data\":{\"createChild\":{\"id\":4}}}")

    val res10 = server.query(
      """
        |mutation {
        |  updateChild(
        |    where: { id: 4 }
        |    data: {
        |      parent: {
        |        upsert: {
        |          create: {
        |            id: 3
        |            p: "Parent3",
        |          }
        |          update: {
        |            p: "doesn't matter"
        |          }
        |        }
        |      }
        |    }
        |  ) {
        |    id
        |    parent {
        |      p
        |      child {
        |        id
        |      }
        |    }
        |  }
        |}
      """,
      project
    )

    res10.toString() should be("{\"data\":{\"updateChild\":{\"id\":4,\"parent\":{\"p\":\"Parent3\",\"child\":{\"id\":4}}}}}")

    val res11 = server.query(
      """
        |mutation {
        |  updateChild(
        |    where: { id: 3 }
        |    data: {
        |      parent: {
        |        delete: true
        |      }
        |    }
        |  ) {
        |    id
        |    parent {
        |      child {
        |        id
        |      }
        |    }
        |  }
        |}
      """,
      project
    )

    res11.toString() should be("{\"data\":{\"updateChild\":{\"id\":3,\"parent\":null}}}")
  }

  // Mutations in this test:
  //  create         | root   | checked
  //  update         | root   | checked
  //  delete         | root   | checked
  //  upsert         | root   | checked
  //  updateMany     | root   | unnecessary
  //  deleteMany     | root   | unnecessary
  //  nested create  | create | checked
  //  nested update  | update | checked
  //  nested connect | create | checked
  //  nested connect | update | checked
  //  nested delete  | -      | checked
  //  nested upsert  | update | checked
  //  nested deleteM | -      | checked
  //  nested updateM | -      | checked
  //  nested disconn | -      | not possible (1!:m)
  //  nested set     | -      | not (really) possible (1!:m)
  "Using an ID that is also a 1!:M single-field relation" should "work" in {
    val project = ProjectDsl.fromString {
      s"""
         |model Parent {
         |  id    Int    @id
         |  child Child  @relation(references: [id]) @unique
         |  p     String
         |}
         |
         |model Child {
         |  id      Int     @id
         |  c       String
         |  parents Parent[]
         |
         |  @@unique([id, c])
         |}
       """
    }
    database.setup(project)

    val res1 = server.query(
      """
        |mutation {
        |  createParent(data: { id: 1, p: "Parent1", child: { create: { id: 1, c: "Child1" }}}) {
        |    p
        |    child {
        |       id
        |       c
        |    }
        |  }
        |}
      """,
      project
    )

    res1.toString() should be("{\"data\":{\"createParent\":{\"p\":\"Parent1\",\"child\":{\"id\":1,\"c\":\"Child1\"}}}}")

    val res2 = server.query(
      """
        |mutation {
        |  updateParent(where: { child: 1 } data: { p: "Parent1Updated" }) {
        |    p
        |  }
        |}
      """,
      project
    )

    res2.toString() should be("{\"data\":{\"updateParent\":{\"p\":\"Parent1Updated\"}}}")

    val res3 = server.query(
      """
        |mutation {
        |  updateChild(where: { id: 1 } data: {
        |    parents: {
        |       updateMany: {
        |         where: { p: "Parent1Updated" }
        |         data: { p: "Parent2UpdatedMany" } }
        |       }
        |     }
        |  ) {
        |    parents {
        |      p
        |    }
        |  }
        |}
      """,
      project
    )

    res3.toString() should be("{\"data\":{\"updateChild\":{\"parents\":[{\"p\":\"Parent2UpdatedMany\"}]}}}")

    val res4 = server.query(
      """
        |mutation {
        |  upsertParent(
        |    where: { child: 2 }
        |    update: { p: "doesn't matter" }
        |    create: { id: 2, p: "Parent2", child: { create: { id: 2, c: "Child2" } } }
        |  ) {
        |    p
        |  }
        |}
      """,
      project
    )

    res4.toString() should be("{\"data\":{\"upsertParent\":{\"p\":\"Parent2\"}}}")

    val res5 = server.query(
      """
        |mutation {
        |  deleteParent(
        |    where: { child: 2 }
        |  ) {
        |    p
        |  }
        |}
      """,
      project
    )

    res5.toString() should be("{\"data\":{\"deleteParent\":{\"p\":\"Parent2\"}}}")

    val res6 = server.query(
      """
        |mutation {
        |  createParent(
        |    data: {
        |      id: 2
        |      p: "Parent2New",
        |      child: {
        |        connect: {
        |          id: 2
        |        }
        |      }
        |    }
        |  ) {
        |    p
        |    child {
        |      id
        |    }
        |  }
        |}
      """,
      project
    )

    res6.toString() should be("{\"data\":{\"createParent\":{\"p\":\"Parent2New\",\"child\":{\"id\":2}}}}")

    val res7 = server.query(
      """
        |mutation {
        |  createChild(
        |    data: {
        |      id: 3
        |      c: "Child3",
        |    }
        |  ) {
        |    id
        |  }
        |}
        |
      """,
      project
    )

    res7.toString() should be("{\"data\":{\"createChild\":{\"id\":3}}}")

    val res8 = server.query(
      """
        |mutation {
        |  updateParent(
        |    where: { child: 2 }
        |    data: {
        |      child: {
        |        connect: {
        |          id: 3
        |        }
        |      }
        |    }
        |  ) {
        |    child {
        |      id
        |    }
        |  }
        |}
      """,
      project
    )

    res8.toString() should be("{\"data\":{\"updateParent\":{\"child\":{\"id\":3}}}}")

    val res9 = server.query(
      """
        |mutation {
        |  createChild(
        |    data: {
        |      id: 4
        |      c: "Child4",
        |    }
        |  ) {
        |    id
        |  }
        |}
        |
      """,
      project
    )

    res9.toString() should be("{\"data\":{\"createChild\":{\"id\":4}}}")

    val res10 = server.query(
      """
        |mutation {
        |  updateChild(
        |    where: { id: 4 }
        |    data: {
        |      parents: {
        |        upsert: {
        |          where: { child: 3 }
        |          create: { id: 3, p: "Parent3" }
        |          update: { p: "doesn't matter" }
        |        }
        |      }
        |    }
        |  ) {
        |    id
        |    parents {
        |      id
        |      child {
        |        id
        |      }
        |    }
        |  }
        |}
      """,
      project
    )

    res10.toString() should be("{\"data\":{\"updateChild\":{\"id\":4,\"parents\":[{\"id\":3,\"child\":{\"id\":4}}]}}}")

    val res11 = server.query(
      """
        |mutation {
        |  updateChild(
        |    where: { id: 3 }
        |    data: {
        |      parents: {
        |        updateMany: {
        |          where: { p: "Parent2New" }
        |          data: { p: "Parent2NewUpdateMany" }
        |        }
        |      }
        |    }
        |  ) {
        |    id
        |    parents {
        |      p
        |      child {
        |        id
        |      }
        |    }
        |  }
        |}
      """,
      project
    )

    res11.toString() should be("{\"data\":{\"updateChild\":{\"id\":3,\"parents\":[{\"p\":\"Parent2NewUpdateMany\",\"child\":{\"id\":3}}]}}}")

    val res12 = server.query(
      """
        |mutation {
        |  updateChild(
        |    where: { id: 3 }
        |    data: {
        |      parents: {
        |        deleteMany: {
        |          p: "Parent2NewUpdateMany"
        |        }
        |      }
        |    }
        |  ) {
        |    id
        |    parents {
        |      child {
        |        id
        |      }
        |    }
        |  }
        |}
      """,
      project
    )

    res12.toString() should be("{\"data\":{\"updateChild\":{\"id\":3,\"parents\":[]}}}")
  }

  // Mutations in this test:
  //  create         | root   | checked
  //  update         | root   | checked
  //  delete         | root   | checked
  //  upsert         | root   | checked
  //  updateMany     | root   | unnecessary
  //  deleteMany     | root   | unnecessary
  //  nested create  | create | checked
  //  nested update  | update | checked
  //  nested connect | create | checked
  //  nested connect | update | checked
  //  nested delete  | -      | checked
  //  nested upsert  | update | checked
  //  nested deleteM | -      | checked
  //  nested updateM | -      | checked
  //  nested disconn | -      | not possible (1!:m)
  //  nested set     | -      | not (really) possible (1!:m)
  "Using an ID that is also a 1!:M multi-field relation" should "work" in {
    val project = ProjectDsl.fromString {
      s"""
         |model Parent {
         |  id    Int    @id
         |  child Child  @relation(references: [id, c]) @unique
         |  p     String
         |}
         |
         |model Child {
         |  id      Int     @id
         |  c       String
         |  parents Parent[]
         |
         |  @@unique([id, c])
         |}
       """
    }
    database.setup(project)

    val res1 = server.query(
      """
        |mutation {
        |  createParent(data: { id: 1, p: "Parent1", child: { create: { id: 1, c: "Child1" }}}) {
        |    p
        |    child {
        |       id
        |       c
        |    }
        |  }
        |}
      """,
      project
    )

    res1.toString() should be("{\"data\":{\"createParent\":{\"p\":\"Parent1\",\"child\":{\"id\":1,\"c\":\"Child1\"}}}}")

    val res2 = server.query(
      """
        |mutation {
        |  updateParent(where: { child: { child_id: 1, child_c: "Child1" } } data: { p: "Parent1Updated" }) {
        |    p
        |  }
        |}
      """,
      project
    )

    res2.toString() should be("{\"data\":{\"updateParent\":{\"p\":\"Parent1Updated\"}}}")

    val res3 = server.query(
      """
        |mutation {
        |  updateChild(where: { id: 1 } data: {
        |    parents: {
        |       updateMany: {
        |         where: { p: "Parent1Updated" }
        |         data: { p: "Parent2UpdatedMany" } }
        |       }
        |     }
        |  ) {
        |    parents {
        |      p
        |    }
        |  }
        |}
      """,
      project
    )

    res3.toString() should be("{\"data\":{\"updateChild\":{\"parents\":[{\"p\":\"Parent2UpdatedMany\"}]}}}")

    val res4 = server.query(
      """
        |mutation {
        |  upsertParent(
        |    where: { child: { child_id: 2, child_c: "Child2" } }
        |    update: { p: "doesn't matter" }
        |    create: { id: 2, p: "Parent2", child: { create: { id: 2, c: "Child2" } } }
        |  ) {
        |    p
        |  }
        |}
      """,
      project
    )

    res4.toString() should be("{\"data\":{\"upsertParent\":{\"p\":\"Parent2\"}}}")

    val res5 = server.query(
      """
        |mutation {
        |  deleteParent(
        |    where: { child: { child_id: 2, child_c: "Child2" } }
        |  ) {
        |    p
        |  }
        |}
      """,
      project
    )

    res5.toString() should be("{\"data\":{\"deleteParent\":{\"p\":\"Parent2\"}}}")

    val res6 = server.query(
      """
        |mutation {
        |  createParent(
        |    data: {
        |      id: 2
        |      p: "Parent2New",
        |      child: {
        |        connect: {
        |          id: 2
        |        }
        |      }
        |    }
        |  ) {
        |    p
        |    child {
        |      id
        |    }
        |  }
        |}
      """,
      project
    )

    res6.toString() should be("{\"data\":{\"createParent\":{\"p\":\"Parent2New\",\"child\":{\"id\":2}}}}")

    val res7 = server.query(
      """
        |mutation {
        |  createChild(
        |    data: {
        |      id: 3
        |      c: "Child3",
        |    }
        |  ) {
        |    id
        |  }
        |}
        |
      """,
      project
    )

    res7.toString() should be("{\"data\":{\"createChild\":{\"id\":3}}}")

    val res8 = server.query(
      """
        |mutation {
        |  updateParent(
        |    where: { child: { child_id: 2, child_c: "Child2" } }
        |    data: {
        |      child: {
        |        connect: {
        |          id: 3
        |        }
        |      }
        |    }
        |  ) {
        |    child {
        |      id
        |    }
        |  }
        |}
      """,
      project
    )

    res8.toString() should be("{\"data\":{\"updateParent\":{\"child\":{\"id\":3}}}}")

    val res9 = server.query(
      """
        |mutation {
        |  createChild(
        |    data: {
        |      id: 4
        |      c: "Child4",
        |    }
        |  ) {
        |    id
        |  }
        |}
        |
      """,
      project
    )

    res9.toString() should be("{\"data\":{\"createChild\":{\"id\":4}}}")

    val res10 = server.query(
      """
        |mutation {
        |  updateChild(
        |    where: { id: 4 }
        |    data: {
        |      parents: {
        |        upsert: {
        |          where: { child: { child_id: 3, child_c: "Child3" } }
        |          create: { id: 3, p: "Parent3" }
        |          update: { p: "doesn't matter" }
        |        }
        |      }
        |    }
        |  ) {
        |    id
        |    parents {
        |      id
        |      child {
        |        id
        |      }
        |    }
        |  }
        |}
      """,
      project
    )

    res10.toString() should be("{\"data\":{\"updateChild\":{\"id\":4,\"parents\":[{\"id\":3,\"child\":{\"id\":4}}]}}}")

    val res11 = server.query(
      """
        |mutation {
        |  updateChild(
        |    where: { id: 3 }
        |    data: {
        |      parents: {
        |        updateMany: {
        |          where: { p: "Parent2New" }
        |          data: { p: "Parent2NewUpdateMany" }
        |        }
        |      }
        |    }
        |  ) {
        |    id
        |    parents {
        |      p
        |      child {
        |        id
        |      }
        |    }
        |  }
        |}
      """,
      project
    )

    res11.toString() should be("{\"data\":{\"updateChild\":{\"id\":3,\"parents\":[{\"p\":\"Parent2NewUpdateMany\",\"child\":{\"id\":3}}]}}}")

    val res12 = server.query(
      """
        |mutation {
        |  updateChild(
        |    where: { id: 3 }
        |    data: {
        |      parents: {
        |        deleteMany: {
        |          p: "Parent2NewUpdateMany"
        |        }
        |      }
        |    }
        |  ) {
        |    id
        |    parents {
        |      child {
        |        id
        |      }
        |    }
        |  }
        |}
      """,
      project
    )

    res12.toString() should be("{\"data\":{\"updateChild\":{\"id\":3,\"parents\":[]}}}")
  }

  // ---------------------------------------
  // Basic cursor tests:
  // - Before
  // - After
  // ---------------------------------------
  "Using cursors for an unique that is a 1!:M single-field relation" should "work" in {
    val project = ProjectDsl.fromString {
      s"""
         |model Parent {
         |  id    Int    @id
         |  child Child  @relation(references: [id]) @unique
         |  p     String
         |}
         |
         |model Child {
         |  id      Int     @id
         |  c       String
         |  parents Parent[]
         |
         |  @@unique([id, c])
         |}
       """
    }
    database.setup(project)

    val p1 = server.query(
      """
        |mutation {
        |  createParent(data: { id: 1, p: "Parent1", child: { create: { id: 1, c: "Child1" }}}) {
        |    p
        |    child {
        |       id
        |       c
        |    }
        |  }
        |}
      """,
      project
    )

    p1.toString() should be("{\"data\":{\"createParent\":{\"p\":\"Parent1\",\"child\":{\"id\":1,\"c\":\"Child1\"}}}}")

    val p2 = server.query(
      """
        |mutation {
        |  createParent(data: { id: 2, p: "Parent2", child: { create: { id: 2, c: "Child2" }}}) {
        |    p
        |    child {
        |       id
        |       c
        |    }
        |  }
        |}
      """,
      project
    )

    p2.toString() should be("{\"data\":{\"createParent\":{\"p\":\"Parent2\",\"child\":{\"id\":2,\"c\":\"Child2\"}}}}")

    val p3 = server.query(
      """
        |mutation {
        |  createParent(data: { id: 3, p: "Parent3", child: { create: { id: 3, c: "Child3" }}}) {
        |    p
        |    child {
        |       id
        |       c
        |    }
        |  }
        |}
      """,
      project
    )

    p3.toString() should be("{\"data\":{\"createParent\":{\"p\":\"Parent3\",\"child\":{\"id\":3,\"c\":\"Child3\"}}}}")

    val beforeCursor = server.query(
      """
        |query {
        |  parents(
        |    before: {
        |      child: 3
        |    }
        |  ){
        |    p
        |    child {
        |       id
        |       c
        |    }
        |  }
        |}
      """,
      project
    )

    beforeCursor.toString() should be(
      "{\"data\":{\"parents\":[{\"p\":\"Parent1\",\"child\":{\"id\":1,\"c\":\"Child1\"}},{\"p\":\"Parent2\",\"child\":{\"id\":2,\"c\":\"Child2\"}}]}}")

    val afterCursor = server.query(
      """
        |query {
        |  parents(
        |    after: {
        |      child: 1
        |    }
        |  ){
        |    p
        |    child {
        |       id
        |       c
        |    }
        |  }
        |}
      """,
      project
    )

    afterCursor.toString() should be(
      "{\"data\":{\"parents\":[{\"p\":\"Parent2\",\"child\":{\"id\":2,\"c\":\"Child2\"}},{\"p\":\"Parent3\",\"child\":{\"id\":3,\"c\":\"Child3\"}}]}}")

    val beforeAfterCursor = server.query(
      """
        |query {
        |  parents(
        |    after: {
        |      child: 1
        |    }
        |    before: {
        |      child: 3
        |    }
        |  ){
        |    p
        |    child {
        |       id
        |       c
        |    }
        |  }
        |}
      """,
      project
    )

    beforeAfterCursor.toString() should be("{\"data\":{\"parents\":[{\"p\":\"Parent2\",\"child\":{\"id\":2,\"c\":\"Child2\"}}]}}")
  }

  // ---------------------------------------
  // Basic cursor tests:
  // - Before
  // - After
  // ---------------------------------------
  "Using cursors for a unique that is a 1!:M multi-field relation" should "work" in {
    val project = ProjectDsl.fromString {
      s"""
         |model Parent {
         |  id    Int    @id
         |  child Child  @relation(references: [id, c]) @unique
         |  p     String
         |}
         |
         |model Child {
         |  id      Int     @id
         |  c       String
         |  parents Parent[]
         |
         |  @@unique([id, c])
         |}
       """
    }
    database.setup(project)

    val p1 = server.query(
      """
        |mutation {
        |  createParent(data: { id: 1, p: "Parent1", child: { create: { id: 1, c: "Child1" }}}) {
        |    p
        |    child {
        |       id
        |       c
        |    }
        |  }
        |}
      """,
      project
    )

    p1.toString() should be("{\"data\":{\"createParent\":{\"p\":\"Parent1\",\"child\":{\"id\":1,\"c\":\"Child1\"}}}}")

    val p2 = server.query(
      """
        |mutation {
        |  createParent(data: { id: 2, p: "Parent2", child: { create: { id: 2, c: "Child2" }}}) {
        |    p
        |    child {
        |       id
        |       c
        |    }
        |  }
        |}
      """,
      project
    )

    p2.toString() should be("{\"data\":{\"createParent\":{\"p\":\"Parent2\",\"child\":{\"id\":2,\"c\":\"Child2\"}}}}")

    val p3 = server.query(
      """
        |mutation {
        |  createParent(data: { id: 3, p: "Parent3", child: { create: { id: 3, c: "Child3" }}}) {
        |    p
        |    child {
        |       id
        |       c
        |    }
        |  }
        |}
      """,
      project
    )

    p3.toString() should be("{\"data\":{\"createParent\":{\"p\":\"Parent3\",\"child\":{\"id\":3,\"c\":\"Child3\"}}}}")

    val beforeCursor = server.query(
      """
        |query {
        |  parents(
        |    before: {
        |      child: {
        |        child_id: 3,
        |        child_c: "Child3"
        |      }
        |    }
        |  ){
        |    p
        |    child {
        |       id
        |       c
        |    }
        |  }
        |}
      """,
      project
    )

    beforeCursor.toString() should be(
      "{\"data\":{\"parents\":[{\"p\":\"Parent1\",\"child\":{\"id\":1,\"c\":\"Child1\"}},{\"p\":\"Parent2\",\"child\":{\"id\":2,\"c\":\"Child2\"}}]}}")

    val afterCursor = server.query(
      """
        |query {
        |  parents(
        |    after: {
        |      child: {
        |        child_id: 1,
        |        child_c: "Child1"
        |      }
        |    }
        |  ){
        |    p
        |    child {
        |       id
        |       c
        |    }
        |  }
        |}
      """,
      project
    )

    afterCursor.toString() should be(
      "{\"data\":{\"parents\":[{\"p\":\"Parent2\",\"child\":{\"id\":2,\"c\":\"Child2\"}},{\"p\":\"Parent3\",\"child\":{\"id\":3,\"c\":\"Child3\"}}]}}")

    val beforeAfterCursor = server.query(
      """
        |query {
        |  parents(
        |    after: {
        |      child: {
        |        child_id: 1,
        |        child_c: "Child1"
        |      }
        |    }
        |    before: {
        |      child: {
        |        child_id: 3,
        |        child_c: "Child3"
        |      }
        |    }
        |  ){
        |    p
        |    child {
        |       id
        |       c
        |    }
        |  }
        |}
      """,
      project
    )

    beforeAfterCursor.toString() should be("{\"data\":{\"parents\":[{\"p\":\"Parent2\",\"child\":{\"id\":2,\"c\":\"Child2\"}}]}}")
  }
}
