package writes.nestedMutations.notUsingSchemaBase

import org.scalatest.{FlatSpec, Matchers}
import util.ConnectorCapability.JoinRelationLinksCapability
import util._

class NestedConnectOrCreateMutationSpec extends FlatSpec with Matchers with ApiSpecBase with SchemaBaseV11 {
  override def runOnlyForCapabilities = Set(JoinRelationLinksCapability)

  "A m:n relation connectOrCreate" should "always work" in {
    val project = SchemaDsl.fromStringV11() {
      """
        |model ModelA {
        |  id    String   @id @default(cuid())
        |  manyB ModelB[]
        |}
        |
        |model ModelB {
        |  id    String   @id @default(cuid())
        |  manyA ModelA[]
        |}
      """.stripMargin
    }
    database.setup(project)

    // Both records are new
    var result = server.query(
      s"""mutation{
           |  createOneModelA(data: {
           |    id: "A1"
           |    manyB: {
           |      connectOrCreate: {
           |        where: { id: "B1" }
           |        create: {
           |          id: "B1"
           |        }
           |      }
           |    }
           |  }) {
           |    id
           |    manyB {
           |      id
           |    }
           |  }
           |}
      """,
      project,
      legacy = false,
    )

    result.toString() should be("{\"data\":{\"createOneModelA\":{\"id\":\"A1\",\"manyB\":[{\"id\":\"B1\"}]}}}")

    // New parent, connect existing child
    result = server.query(
      s"""mutation{
         |  createOneModelA(data: {
         |    id: "A2"
         |    manyB: {
         |      connectOrCreate: {
         |        where: { id: "B1" }
         |        create: {
         |          id: "Doesn't matter"
         |        }
         |      }
         |    }
         |  }) {
         |    id
         |    manyB {
         |      id
         |    }
         |  }
         |}
      """,
      project,
      legacy = false,
    )

    result.toString() should be("{\"data\":{\"createOneModelA\":{\"id\":\"A2\",\"manyB\":[{\"id\":\"B1\"}]}}}")

    // Update a parent to connect a new child
    result = server.query(
      s"""mutation{
         |  updateOneModelA(
         |    where: { id: "A1" }
         |    data: {
         |      manyB: {
         |        connectOrCreate: {
         |          where: { id: "B2" }
         |          create: {
         |            id: "B2"
         |          }
         |        }
         |      }
         |    }
         |  ) {
         |    id
         |    manyB {
         |      id
         |    }
         |  }
         |}
      """,
      project,
      legacy = false,
    )

    result.toString() should be("{\"data\":{\"updateOneModelA\":{\"id\":\"A1\",\"manyB\":[{\"id\":\"B1\"},{\"id\":\"B2\"}]}}}")
  }

  "A 1!:m relation connectOrCreate" should "work and prevent relation violations" in {}

  "A 1:m relation connectOrCreate with the one side optional" should "work" in {}

  "A 1!:1! relation connectOrCreate" should "work and prevent relation violations" in {}

  "A 1:1! relation connectOrCreate" should "work and prevent relation violations" in {}

  "A 1:1 relation connectOrCreate" should "work" in {}
}
